use helpers::{
    Resolve,
    SyncRef,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use language::{
    BOOLEAN_TYPE,
    Expression,
    ExpressionAST,
    DataType,
    DataTypeAST,
    Deleting,
    Inserting,
    ItemPath,
    Selection,
    SelectionAST,
    Updating,
};
use project_analysis::{
    FunctionVariable,
    FunctionVariableScope,
    SemanticError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CycleTypeAST<'source> {
    Simple,
    PrePredicated(ExpressionAST<'source>),
    PostPredicated(ExpressionAST<'source>),
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for CycleTypeAST<'source> {
    type Result = CycleType;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let result = match self {
            CycleTypeAST::Simple => CycleType::Simple,
            CycleTypeAST::PrePredicated(predicate) => CycleType::PrePredicated(predicate.resolve(scope)?),
            CycleTypeAST::PostPredicated(predicate) => CycleType::PostPredicated(predicate.resolve(scope)?),
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CycleType {
    Simple,
    PrePredicated(Expression),
    PostPredicated(Expression),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleControlOperator {
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementSourceAST<'source> {
    Expression(ExpressionAST<'source>),
    Selection(SelectionAST<'source>),
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for StatementSourceAST<'source> {
    type Result = StatementSource;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let result = match self {
            StatementSourceAST::Expression(expr) => StatementSource::Expression(expr.resolve(scope)?),
            StatementSourceAST::Selection(select) => StatementSource::Selection(select.resolve(scope)?)
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementSource {
    Expression(Expression),
    Selection(Selection),
}

impl StatementSource {
    pub fn type_of(&self) -> &DataType {
        match self {
            StatementSource::Expression(expr) => &expr.data_type,
            StatementSource::Selection(query) => &query.result_data_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementASTBody<'source> {
    VariableDefinition {
        name: Identifier<'source>,
        data_type: Option<DataTypeAST<'source>>,
        default_value: Option<StatementSourceAST<'source>>,
    },
    VariableAssignment {
        path: ItemPath,
        source: StatementSourceAST<'source>,
    },
    Condition {
        condition: ExpressionAST<'source>,
        then_body: Box<StatementAST<'source>>,
        else_body: Option<Box<StatementAST<'source>>>,
    },
    Cycle {
        cycle_type: CycleTypeAST<'source>,
        body: Box<StatementAST<'source>>,
    },
    CycleControl {
        operator: CycleControlOperator,
        name: Option<Identifier<'source>>,
    },
    Return {
        value: Option<StatementSourceAST<'source>>,
    },
    Block {
        statements: Vec<StatementAST<'source>>,
    },
    Expression {
        expression: ExpressionAST<'source>,
    },
    DeletingRequest {
        request: Deleting<'source>,
    },
    InsertingRequest {
        request: Inserting<'source>,
    },
    UpdatingRequest {
        request: Updating<'source>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatementAST<'source> {
    pub body: StatementASTBody<'source>,
    pub pos: ItemPosition,
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for StatementAST<'source> {
    type Result = Statement;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        match &self.body {
            StatementASTBody::VariableDefinition { name, data_type, default_value } => {
                let data_type = data_type.resolve(&ctx.context().module())?;
                let default_value: Option<StatementSource> = default_value.resolve(ctx)?;
                let data_type = match data_type {
                    Some(data_type) => {
                        if let &Some(ref default_value) = &default_value {
                            let default_value_type = default_value.type_of();
                            default_value_type.should_cast_to(self.pos, &data_type)?;
                        }
                        Some(data_type)
                    }
                    None => {
                        match &default_value {
                            &Some(ref default_value) => Some(default_value.type_of().clone()),
                            &None => None,
                        }
                    }
                };
                let var = ctx.new_variable(name.item_pos(), name.to_string(), data_type)?;
                let stmt = match default_value {
                    Some(source) => Statement::VariableAssignment {
                        var,
                        source,
                    },
                    None => Statement::Nothing,
                };
                Ok(stmt)
            }
            StatementASTBody::VariableAssignment { path, source } => {
                let mut var_path = path.path.as_path();
                let name = var_path.pop_left()
                    .expect("Assignment's target path should not be empty");
                let source = source.resolve(ctx)?;
                let var = ctx.access_to_variable(self.pos, name)?;
                if var.is_read_only() {
                    return SemanticError::cannot_modify_readonly_variable(self.pos, name.to_string())
                        .into_err_vec();
                }
                {
                    let source_type = source.type_of();
                    if var_path.is_empty() {
                        match var.read().data_type() {
                            Some(var_type) => {
                                source_type.should_cast_to(self.pos, var_type)?;
                            }
                            None => {
                                var.replace_data_type(source_type.clone());
                            }
                        }
                    } else {
                        let prop_type = var.property_type(&ItemPath {
                            pos: path.pos,
                            path: var_path.into(),
                        })?;
                        source_type.should_cast_to(self.pos, &prop_type)?;
                    }
                }
                Ok(Statement::VariableAssignment {
                    var,
                    source,
                })
            }
            StatementASTBody::Condition { condition, then_body, else_body } => {
                let mut errors = Vec::new();
                let condition = condition.accumulative_resolve(ctx, &mut errors);
                let then_body = then_body.accumulative_resolve(ctx, &mut errors);
                let else_body = else_body.accumulative_resolve(ctx, &mut errors);
                let condition = match condition {
                    Some(x) => x,
                    None => return Err(errors),
                };
                condition.should_cast_to_type(&BOOLEAN_TYPE)?;
                let then_body = match then_body {
                    Some(x) => x,
                    None => return Err(errors),
                };
                let else_body = match else_body {
                    Some(x) => x,
                    None => return Err(errors),
                };
                Ok(Statement::Condition {
                    condition,
                    then_body,
                    else_body,
                })
            }
            StatementASTBody::Cycle { cycle_type, body } => {
                let mut errors = Vec::new();
                let cycle_type = cycle_type.accumulative_resolve(ctx, &mut errors);
                let body = body.accumulative_resolve(ctx, &mut errors);
                let cycle_type = match cycle_type {
                    Some(x) => x,
                    None => return Err(errors),
                };
                match &cycle_type {
                    CycleType::PostPredicated(predicate) => predicate.should_cast_to_type(&BOOLEAN_TYPE)?,
                    CycleType::PrePredicated(predicate) => predicate.should_cast_to_type(&BOOLEAN_TYPE)?,
                    CycleType::Simple => {}
                }
                let body = match body {
                    Some(x) => x,
                    None => return Err(errors),
                };
                Ok(Statement::Cycle {
                    cycle_type,
                    body,
                })
            }
            StatementASTBody::CycleControl { operator, name } => {
                if name.is_some() {
                    return SemanticError::not_supported_yet(self.pos, "cycle control labels")
                        .into_err_vec();
                }
                Ok(Statement::CycleControl {
                    operator: *operator,
                })
            }
            StatementASTBody::Return { value } => {
                let value = value.resolve(ctx)?;
                Ok(Statement::Return {
                    value,
                })
            }
            StatementASTBody::Block { statements } => {
                let scope = ctx.child();
                let mut result = Vec::with_capacity(statements.len());
                for statement in statements {
                    result.push(statement.resolve(&scope)?);
                }
                Ok(Statement::Block {
                    statements: result,
                })
            }
            StatementASTBody::Expression { expression } => {
                let expression = expression.resolve(ctx)?;
                Ok(Statement::Expression {
                    expression,
                })
            }
            _ => unimplemented!()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Nothing,
    VariableAssignment {
        var: SyncRef<FunctionVariable>,
        source: StatementSource,
    },
    Condition {
        condition: Expression,
        then_body: Box<Statement>,
        else_body: Option<Box<Statement>>,
    },
    Cycle {
        cycle_type: CycleType,
        body: Box<Statement>,
    },
    CycleControl {
        operator: CycleControlOperator,
    },
    Return {
        value: Option<StatementSource>,
    },
    Block {
        statements: Vec<Statement>,
    },
    Expression {
        expression: Expression,
    },
}

impl Statement {
    pub fn is_lite_weight(&self) -> bool {
        match self {
            Statement::Nothing => true,
            Statement::VariableAssignment { var: _, source: _ } => true,
            Statement::Condition { condition, then_body, else_body } => {
                let is_else_body_lite_weight = match else_body {
                    Some(body) => body.is_lite_weight(),
                    None => true
                };
                is_else_body_lite_weight
                    && condition.is_lite_weight()
                    && then_body.is_lite_weight()
            }
            Statement::Cycle { cycle_type, body } => {
                let is_predicate_lite_weight = match cycle_type {
                    CycleType::Simple => true,
                    CycleType::PostPredicated(predicate) => predicate.is_lite_weight(),
                    CycleType::PrePredicated(predicate) => predicate.is_lite_weight(),
                };
                is_predicate_lite_weight
                    && body.is_lite_weight()
            }
            Statement::CycleControl { operator: _ } => true,
            Statement::Return { value } => match value {
                Some(StatementSource::Expression(expr)) => expr.is_lite_weight(),
                Some(StatementSource::Selection(_)) => true,
                None => true,
            },
            Statement::Block { statements } => statements.iter()
                .all(|stmt| stmt.is_lite_weight()),
            Statement::Expression { expression } => expression.is_lite_weight(),
        }
    }
    //TODO Все ветви кода должны возвращать значение корректного типа.
    //TODO Выражения типа, отличного от Void, должны сохранять результат своего выполнения.
}
