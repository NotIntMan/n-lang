use syntax_parser::compound_types::DataType;
use syntax_parser::expressions::Expression;

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
pub enum Statement<'source> {
    Nothing,
    VariableDefinition {
        name: &'source str,
        data_type: Option<DataType<'source>>,
        default_value: Option<Expression<'source>>,
    },
    VariableAssignment {
        name: &'source str,
        expression: Expression<'source>,
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
        value: Option<Expression<'source>>,
    },
    Block {
        statements: Vec<Statement<'source>>,
    },
    Expression {
        expression: Expression<'source>,
    },
}

impl<'source> Default for Statement<'source> {
    fn default() -> Self { Statement::Nothing }
}

// TODO Не забыть про запросы манипуляции в телах функции. Скорее всего, они станут высказываниями или выражениями (но лучше - высказываниями).
