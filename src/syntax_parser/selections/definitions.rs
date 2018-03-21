use parser_basics::Identifier;
use syntax_parser::expressions::Expression;
use syntax_parser::data_sources::DataSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionResultSize {
    Usual,
    Small,
    Big,
    Buffered,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionExpression<'source> {
    pub expr: Expression<'source>,
    pub alias: Option<Identifier<'source>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionResult<'source> {
    All,
    Some(Vec<SelectionExpression<'source>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionSortingOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSortingItem<'source> {
    pub expr: Expression<'source>,
    pub order: SelectionSortingOrder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionGroupByClause<'source> {
    pub sorting: Vec<SelectionSortingItem<'source>>,
    pub with_rollup: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionLimit {
    pub offset: Option<u32>,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection<'source> {
    pub distinct: bool,
    pub high_priority: bool,
    pub straight_join: bool,
    pub result_size: SelectionResultSize,
    pub cache: bool,
    pub result: SelectionResult<'source>,
    pub source: DataSource<'source>,
    pub where_clause: Option<Expression<'source>>,
    pub group_by_clause: Option<SelectionGroupByClause<'source>>,
    pub having_clause: Option<Expression<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItem<'source>>>,
    pub limit_clause: Option<SelectionLimit>,
}
