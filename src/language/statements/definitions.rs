//use helpers::IntoStatic;
use helpers::{
    Resolve,
    SyncRef,
};
use parser_basics::Identifier;
use language::{
    Expression,
    ExpressionAST,
    DataType,
    DataTypeAST,
    Deleting,
    Inserting,
    FunctionVariableScope,
    TableDefinition,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementResultType {
    Data(DataType),
    Table(TableDefinition),
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
pub enum StatementAST<'source> {
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

impl<'source> Default for StatementAST<'source> {
    fn default() -> Self { StatementAST::Nothing }
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

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for StatementAST<'source> {
    type Result = Statement;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        match self {
            &StatementAST::Nothing => Ok(Statement::Nothing),
            &StatementAST::VariableDefinition { ref name, ref data_type, ref default_value } => {
                let data_type = data_type.resolve(&ctx.context().module())?;
                let data_type = match default_value {
                    // TODO Typeof expression/selection
                    &Some(ref _expr) => unimplemented!(),
                    &None => data_type.map(|t| StatementResultType::Data(t)),
                };
                ctx.new_variable(name.item_pos(), name.to_string(), data_type)
                    .map_err(|e| vec![e])?;
                Ok(Statement::Nothing)
            }
            _ => unimplemented!()
        }
    }
}

pub enum Statement {
    Nothing,
}
