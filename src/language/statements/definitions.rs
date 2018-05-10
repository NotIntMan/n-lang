//use helpers::IntoStatic;
use helpers::{
    Resolve,
    SyncRef,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use language::{
    Expression,
    ExpressionAST,
    DataType,
    DataTypeAST,
    Deleting,
    Inserting,
    FunctionVariable,
    FunctionVariableScope,
    Selection,
    Updating,
};
use project_analysis::SemanticError;

#[derive(Debug, Clone, PartialEq)]
pub enum CycleType<'source> {
    Simple,
    PrePredicated(ExpressionAST<'source>),
    PostPredicated(ExpressionAST<'source>),
}

//impl<'source> IntoStatic for CycleType<'source> {
//    type Result = CycleType<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            CycleType::Simple => CycleType::Simple,
//            CycleType::PrePredicated(expr) => CycleType::PrePredicated(expr.into_static()),
//            CycleType::PostPredicated(expr) => CycleType::PostPredicated(expr.into_static()),
//        }
//    }
//}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleControlOperator {
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementSourceAST<'source> {
    Expression(ExpressionAST<'source>),
    Selection(Selection<'source>),
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for StatementSourceAST<'source> {
    type Result = StatementSource;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let result = match self {
            &StatementSourceAST::Expression(ref expr) => StatementSource::Expression(expr.resolve(scope)?),
            _ => unimplemented!(),
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementSource {
    Expression(Expression),
//    Selection(Selection<'source>),
}

impl StatementSource {
    pub fn type_of(&self) -> &DataType {
        match self {
            &StatementSource::Expression(ref expr) => &expr.data_type,
        }
    }
}

//impl<'source> IntoStatic for StatementSource<'source> {
//    type Result = StatementSource<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            StatementSource::Expression(value) => StatementSource::Expression(value.into_static()),
//            StatementSource::Selection(value) => StatementSource::Selection(value.into_static()),
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementASTBody<'source> {
    Nothing,
    VariableDefinition {
        name: Identifier<'source>,
        data_type: Option<DataTypeAST<'source>>,
        default_value: Option<StatementSourceAST<'source>>,
    },
    VariableAssignment {
        name: Identifier<'source>,
        source: StatementSourceAST<'source>,
    },
    Condition {
        condition: ExpressionAST<'source>,
        then_body: Box<StatementAST<'source>>,
        else_body: Option<Box<StatementAST<'source>>>,
    },
    Cycle {
        cycle_type: CycleType<'source>,
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

impl<'source> Default for StatementASTBody<'source> {
    fn default() -> Self { StatementASTBody::Nothing }
}

//impl<'source> IntoStatic for Statement<'source> {
//    type Result = Statement<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            Statement::Nothing => Statement::Nothing,
//            Statement::VariableDefinition { name, data_type, default_value } => Statement::VariableDefinition {
//                name: name.into_static(),
//                data_type: data_type.into_static(),
//                default_value: default_value.into_static(),
//            },
//            Statement::VariableAssignment { name, source } => Statement::VariableAssignment {
//                name: name.into_static(),
//                source: source.into_static(),
//            },
//            Statement::Condition { condition, then_body, else_body } => Statement::Condition {
//                condition: condition.into_static(),
//                then_body: then_body.into_static(),
//                else_body: else_body.into_static(),
//            },
//            Statement::Cycle { cycle_type, body } => Statement::Cycle {
//                cycle_type: cycle_type.into_static(),
//                body: body.into_static(),
//            },
//            Statement::CycleControl { operator, name } => Statement::CycleControl {
//                operator,
//                name: name.into_static(),
//            },
//            Statement::Return { value } => Statement::Return { value: value.into_static() },
//            Statement::Block { statements } => Statement::Block { statements: statements.into_static() },
//            Statement::Expression { expression } => Statement::Expression { expression: expression.into_static() },
//            Statement::DeletingRequest { request } => Statement::DeletingRequest { request: request.into_static() },
//            Statement::InsertingRequest { request } => Statement::InsertingRequest { request: request.into_static() },
//            Statement::UpdatingRequest { request } => Statement::UpdatingRequest { request: request.into_static() },
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatementAST<'source> {
    pub body: StatementASTBody<'source>,
    pub pos: ItemPosition,
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for StatementAST<'source> {
    type Result = Statement;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        match &self.body {
            &StatementASTBody::Nothing => Ok(Statement::Nothing),
            &StatementASTBody::VariableDefinition { ref name, ref data_type, ref default_value } => {
                let data_type = data_type.resolve(&ctx.context().module())?;
                let default_value: Option<StatementSource> = default_value.resolve(ctx)?;
                let data_type = match data_type {
                    Some(data_type) => {
                        if let &Some(ref default_value) = &default_value {
                            let default_value_type = default_value.type_of();
                            if !default_value_type.can_cast(&data_type) {
                                return Err(vec![SemanticError::cannot_cast_type(
                                    self.pos,
                                    data_type,
                                    default_value_type.clone(),
                                )]);
                            }
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
                let var = ctx.new_variable(name.item_pos(), name.to_string(), data_type)
                    .map_err(|e| vec![e])?;
                let stmt = match default_value {
                    Some(source) => Statement::VariableAssignment {
                        var,
                        source,
                    },
                    None => Statement::Nothing,
                };
                Ok(stmt)
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
}

impl Statement {
    pub fn has_side_effects(&self) -> bool {
        match self {
            &Statement::Nothing => false,
            &Statement::VariableAssignment { var: _, source: _ } => false,
        }
    }
}
