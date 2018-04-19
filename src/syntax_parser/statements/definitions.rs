//use helpers::into_static::IntoStatic;
use parser_basics::Identifier;
use syntax_parser::compound_types::DataTypeAST;
use syntax_parser::expressions::Expression;
use syntax_parser::selections::Selection;
use syntax_parser::other_requests::{
    Deleting,
    Inserting,
    Updating,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CycleType<'source> {
    Simple,
    PrePredicated(Expression<'source>),
    PostPredicated(Expression<'source>),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatementSource<'source> {
    Expression(Expression<'source>),
    Selection(Selection<'source>),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<'source> {
    Nothing,
    VariableDefinition {
        name: Identifier<'source>,
        data_type: Option<DataTypeAST<'source>>,
        default_value: Option<StatementSource<'source>>,
    },
    VariableAssignment {
        name: Identifier<'source>,
        source: StatementSource<'source>,
    },
    Condition {
        condition: Expression<'source>,
        then_body: Box<Statement<'source>>,
        else_body: Option<Box<Statement<'source>>>,
    },
    Cycle {
        cycle_type: CycleType<'source>,
        body: Box<Statement<'source>>,
    },
    CycleControl {
        operator: CycleControlOperator,
        name: Option<Identifier<'source>>,
    },
    Return {
        value: Option<StatementSource<'source>>,
    },
    Block {
        statements: Vec<Statement<'source>>,
    },
    Expression {
        expression: Expression<'source>,
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

impl<'source> Default for Statement<'source> {
    fn default() -> Self { Statement::Nothing }
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

impl Statement<'static> {
    pub fn is_resolved(&self) -> bool {
        match self {
            &Statement::Nothing => true,
            _ => unimplemented!(),
        }
    }
}
