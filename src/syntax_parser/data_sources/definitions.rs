use parser_basics::Identifier;
use syntax_parser::expressions::Expression;
use syntax_parser::selections::Selection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinCondition<'source> {
    Expression(Expression<'source>),
    Using(Vec<Vec<Identifier<'source>>>),
    Natural,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Cross,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSource<'source> {
    Table {
        name: Vec<Identifier<'source>>,
        alias: Option<Identifier<'source>>,
    },
    Join {
        join_type: JoinType,
        condition: Option<JoinCondition<'source>>,
        left: Box<DataSource<'source>>,
        right: Box<DataSource<'source>>,
    },
    Selection {
        query: Box<Selection<'source>>,
        alias: Identifier<'source>,
    },
}
