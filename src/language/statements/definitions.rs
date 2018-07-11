use helpers::{
    BlockFormatter,
    Path,
    PathBuf,
    Resolve,
    SyncRef,
};
use language::{
    BOOLEAN_TYPE,
    DataType,
    DataTypeAST,
    Deleting,
    DeletingAST,
    Expression,
    ExpressionAST,
    Inserting,
    InsertingAST,
    ItemPath,
    Selection,
    SelectionAST,
    TSQLFunctionContext,
    Updating,
    UpdatingAST,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use project_analysis::{
    FunctionVariable,
    FunctionVariableScope,
    SemanticError,
    StatementFlowControlJumping,
    StatementFlowControlPosition,
};
use std::fmt;

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
        request: DeletingAST<'source>,
    },
    InsertingRequest {
        request: InsertingAST<'source>,
    },
    UpdatingRequest {
        request: UpdatingAST<'source>,
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
        let body = match &self.body {
            StatementASTBody::VariableDefinition { name, data_type, default_value } => {
                let data_type = data_type.resolve(&ctx.context().module())?;
                let default_value: Option<StatementSource> = default_value.resolve(ctx)?;
                let var = ctx.new_variable(name.item_pos(), name.to_string(), data_type)?;
                match default_value {
                    Some(source) => {
                        let target = AssignmentTarget::new(
                            var,
                            self.pos,
                            PathBuf::empty(),
                        );
                        target.check_source_type(source.type_of())?;
                        StatementBody::VariableAssignment {
                            target,
                            source,
                        }
                    }
                    None => StatementBody::Nothing,
                }
            }
            StatementASTBody::VariableAssignment { path, source } => {
                let source = source.resolve(ctx)?;
                let target = AssignmentTarget::new_in_scope(
                    ctx,
                    self.pos,
                    path.path.as_path(),
                )?;
                target.check_source_type(source.type_of())?;
                StatementBody::VariableAssignment {
                    target,
                    source,
                }
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
                StatementBody::Condition {
                    condition,
                    then_body,
                    else_body,
                }
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
                StatementBody::Cycle {
                    cycle_type,
                    body,
                }
            }
            StatementASTBody::CycleControl { operator, name } => {
                if name.is_some() {
                    return SemanticError::not_supported_yet(self.pos, "cycle control labels")
                        .into_err_vec();
                }
                StatementBody::CycleControl {
                    operator: *operator,
                }
            }
            StatementASTBody::Return { value } => {
                let value = value.resolve(ctx)?;
                StatementBody::Return {
                    value,
                }
            }
            StatementASTBody::Block { statements } => {
                let scope = ctx.child();
                let mut result = Vec::with_capacity(statements.len());
                for statement in statements {
                    result.push(statement.resolve(&scope)?);
                }
                StatementBody::Block {
                    statements: result,
                }
            }
            StatementASTBody::Expression { expression } => {
                let expression = expression.resolve(ctx)?;
                StatementBody::Expression {
                    expression,
                }
            }
            StatementASTBody::DeletingRequest { request } => {
                let request = request.resolve(ctx)?;
                StatementBody::DeletingRequest { request }
            }
            StatementASTBody::InsertingRequest { request } => {
                let request = request.resolve(ctx)?;
                StatementBody::InsertingRequest { request }
            }
            StatementASTBody::UpdatingRequest { request } => {
                let request = request.resolve(ctx)?;
                StatementBody::UpdatingRequest { request }
            }
        };
        Ok(Statement {
            body,
            pos: self.pos,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentTarget {
    pub var: SyncRef<FunctionVariable>,
    pub property: PathBuf,
    pub pos: ItemPosition,
}

impl AssignmentTarget {
    pub fn new(var: SyncRef<FunctionVariable>, pos: ItemPosition, property: PathBuf) -> Self {
        AssignmentTarget {
            var,
            property,
            pos,
        }
    }
    pub fn new_in_scope(scope: &SyncRef<FunctionVariableScope>, pos: ItemPosition, mut property_path: Path) -> Result<Self, SemanticError> {
        let name = property_path.pop_left()
            .expect("Assignment's target path should not be empty");
        let var = scope.access_to_variable(pos, name)?;
        if var.is_read_only() {
            return Err(SemanticError::cannot_modify_readonly_variable(pos, name.to_string()));
        }
        Ok(AssignmentTarget::new(var, pos, property_path.into()))
    }
    pub fn check_source_type(&self, source_type: &DataType) -> Result<(), SemanticError> {
        let property = self.property.as_path();
        if property.is_empty() {
            let var = self.var.read();
            match var.data_type() {
                Some(var_type) => source_type.should_cast_to(self.pos, var_type)?,
                None => self.var.replace_data_type(source_type.clone()),
            }
        } else {
            source_type.should_cast_to(self.pos, &self.var.property_type(self.pos, property)?)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementBody {
    Nothing,
    VariableAssignment {
        target: AssignmentTarget,
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
    DeletingRequest {
        request: Deleting,
    },
    InsertingRequest {
        request: Inserting,
    },
    UpdatingRequest {
        request: Updating,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub body: StatementBody,
    pub pos: ItemPosition,
}

impl Statement {
    pub fn is_lite_weight(&self) -> bool {
        match &self.body {
            StatementBody::Nothing => true,
            StatementBody::VariableAssignment { target: _, source: _ } => true,
            StatementBody::Condition { condition, then_body, else_body } => {
                let is_else_body_lite_weight = match else_body {
                    Some(body) => body.is_lite_weight(),
                    None => true
                };
                is_else_body_lite_weight
                    && condition.is_lite_weight()
                    && then_body.is_lite_weight()
            }
            StatementBody::Cycle { cycle_type, body } => {
                let is_predicate_lite_weight = match cycle_type {
                    CycleType::Simple => true,
                    CycleType::PostPredicated(predicate) => predicate.is_lite_weight(),
                    CycleType::PrePredicated(predicate) => predicate.is_lite_weight(),
                };
                is_predicate_lite_weight
                    && body.is_lite_weight()
            }
            StatementBody::CycleControl { operator: _ } => true,
            StatementBody::Return { value } => match value {
                Some(StatementSource::Expression(expr)) => expr.is_lite_weight(),
                Some(StatementSource::Selection(_)) => true,
                None => true,
            },
            StatementBody::Block { statements } => statements.iter()
                .all(|stmt| stmt.is_lite_weight()),
            StatementBody::Expression { expression } => expression.is_lite_weight(),
            StatementBody::DeletingRequest { request } => request.is_lite_weight(),
            StatementBody::InsertingRequest { request } => request.is_lite_weight(),
            StatementBody::UpdatingRequest { request } => request.is_lite_weight(),
        }
    }
    //TODO Выражения типа, отличного от Void, должны сохранять результат своего выполнения.
    pub fn jumping_check(&self, pos: StatementFlowControlPosition, return_data_type: &DataType) -> Result<StatementFlowControlJumping, Vec<SemanticError>> {
        match &self.body {
            StatementBody::VariableAssignment { target: _, source: _ } => Ok(StatementFlowControlJumping::Nothing),
            StatementBody::Condition { condition: _, then_body, else_body } => {
                match then_body.jumping_check(pos, return_data_type) {
                    Ok(then_body_jumping) => {
                        let else_body_jumping = match else_body {
                            Some(else_body) => else_body.jumping_check(pos, return_data_type)?,
                            None => StatementFlowControlJumping::Nothing,
                        };
                        Ok(then_body_jumping + else_body_jumping)
                    }
                    Err(mut then_body_errors) => {
                        if let Some(else_body) = else_body {
                            if let Err(mut else_body_errors) = else_body.jumping_check(pos, return_data_type) {
                                then_body_errors.append(&mut else_body_errors);
                            }
                        }
                        Err(then_body_errors)
                    }
                }
            }
            StatementBody::Cycle { cycle_type: _, body } => {
                body.jumping_check(pos.in_cycle(), return_data_type)
            }
            StatementBody::CycleControl { operator } => {
                if !pos.is_in_cycle() {
                    return SemanticError::not_allowed_here(self.pos, "cycle control operators")
                        .into_err_vec();
                }
                match operator {
                    CycleControlOperator::Break => Ok(StatementFlowControlJumping::AlwaysBreaks),
                    CycleControlOperator::Continue => Ok(StatementFlowControlJumping::AlwaysContinues),
                }
            }
            StatementBody::Return { value } => {
                match value {
                    Some(value) => value.type_of().should_cast_to(self.pos, return_data_type)?,
                    None => DataType::Void.should_cast_to(self.pos, return_data_type)?,
                }
                Ok(StatementFlowControlJumping::AlwaysReturns)
            }
            StatementBody::Block { statements } => {
                let mut result = StatementFlowControlJumping::Nothing;
                let mut errors = Vec::new();
                let mut statements_iter = statements.iter();
                while let Some(statement) = statements_iter.next() {
                    match statement.jumping_check(pos, return_data_type) {
                        Ok(local_result) => match local_result {
                            StatementFlowControlJumping::AlwaysReturns |
                            StatementFlowControlJumping::AlwaysBreaks |
                            StatementFlowControlJumping::AlwaysContinues => return match statements_iter.next() {
                                Some(statement) => SemanticError::unreachable_statement(statement.pos).into_err_vec(),
                                None => Ok(local_result),
                            },
                            local_result => if errors.is_empty() {
                                result += local_result;
                            }
                        }
                        Err(mut local_errors) => {
                            errors.append(&mut local_errors);
                        }
                    }
                }
                if errors.is_empty() {
                    Ok(result)
                } else {
                    Err(errors)
                }
            }
            StatementBody::Expression { expression: _ } |
            StatementBody::DeletingRequest { request: _ } |
            StatementBody::InsertingRequest { request: _ } |
            StatementBody::UpdatingRequest { request: _ } |
            StatementBody::Nothing => Ok(StatementFlowControlJumping::Nothing),
        }
    }
    pub fn fmt(
        &self,
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        unimplemented!()
    }
}
