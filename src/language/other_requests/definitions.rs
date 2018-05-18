use helpers::{
    Assertion,
    Resolve,
    SyncRef,
};
use lexeme_scanner::ItemPosition;
use language::{
    DataSourceAST,
    Expression,
    ExpressionAST,
    ItemPath,
    SelectionAST,
    SelectionSortingItemAST,
};
use project_analysis::{
    FunctionVariableScope,
    SemanticError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum UpdatingValueAST<'source> {
    Default(ItemPosition),
    Expression(ExpressionAST<'source>),
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for UpdatingValueAST<'source> {
    type Result = Expression;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        match self {
            UpdatingValueAST::Default(pos) =>
                return SemanticError::not_supported_yet(*pos, "default type's value")
                    .into_err_vec(),
            UpdatingValueAST::Expression(expr) => expr.resolve(scope),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatingAssignmentAST<'source> {
    pub property: ItemPath,
    pub value: UpdatingValueAST<'source>,
}

impl<'a, 'b, 'source> Assertion<(&'a str, Option<&'b str>)> for UpdatingAssignmentAST<'source> {
    fn assert(&self, other: &(&str, Option<&str>)) {
        let other_property_tokens = ::lexeme_scanner::Scanner::scan(other.0)
            .expect("Scanner result must be ok");
        let other_property = ::parser_basics::parse(other_property_tokens.as_slice(), ::language::others::property_path)
            .expect("Parser result must be ok");
        assert_eq!(self.property.path, other_property.path);
        match other.1 {
            Some(other_expr) => {
                match_it!(&self.value, UpdatingValueAST::Expression(expr) => {
                    expr.assert(other_expr)
                });
            },
            None => match_it!(&self.value, UpdatingValueAST::Default(_) => {}),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatingAST<'source> {
    pub low_priority: bool,
    pub ignore: bool,
    pub source: DataSourceAST<'source>,
    pub assignments: Vec<UpdatingAssignmentAST<'source>>,
    pub where_clause: Option<ExpressionAST<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItemAST<'source>>>,
    pub limit_clause: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertingPriority {
    Usual,
    Low,
    Delayed,
    High,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InsertingSourceAST<'source> {
    ValueLists {
        properties: Option<Vec<ItemPath>>,
        lists: Vec<Vec<ExpressionAST<'source>>>,
    },
    AssignmentList {
        assignments: Vec<UpdatingAssignmentAST<'source>>,
    },
    Selection {
        properties: Option<Vec<ItemPath>>,
        query: SelectionAST<'source>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertingAST<'source> {
    pub priority: InsertingPriority,
    pub ignore: bool,
    pub target: DataSourceAST<'source>,
    pub source: InsertingSourceAST<'source>,
    pub on_duplicate_key_update: Option<Vec<UpdatingAssignmentAST<'source>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeletingAST<'source> {
    pub low_priority: bool,
    pub quick: bool,
    pub ignore: bool,
    pub source: DataSourceAST<'source>,
    pub where_clause: Option<ExpressionAST<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItemAST<'source>>>,
    pub limit_clause: Option<u32>,
}
