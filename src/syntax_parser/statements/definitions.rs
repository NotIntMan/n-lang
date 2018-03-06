use syntax_parser::compound_types::DataType;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement<'source> {
    Nothing,
    VariableDefinition {
        name: &'source str,
        data_type: Option<DataType<'source>>,
        default_value: Option<StatementSource<'source>>,
    },
    VariableAssignment {
        name: &'source str,
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
        name: Option<&'source str>,
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
//    TODO Запросы манипуляции как высказывания
//
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
