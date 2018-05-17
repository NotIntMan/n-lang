use helpers::Assertion;
use language::{
    DataSourceAST,
    ExpressionAST,
    ItemPath,
    SelectionAST,
    SelectionSortingItemAST,
};

#[derive(Debug, Clone, PartialEq)]
pub enum UpdatingValue<'source> {
    Default,
    Expression(ExpressionAST<'source>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatingAssignment<'source> {
    pub property: ItemPath,
    pub value: UpdatingValue<'source>,
}

impl<'a, 'b, 'source> Assertion<(&'a str, Option<&'b str>)> for UpdatingAssignment<'source> {
    fn assert(&self, other: &(&str, Option<&str>)) {
        let other_property_tokens = ::lexeme_scanner::Scanner::scan(other.0)
            .expect("Scanner result must be ok");
        let other_property = ::parser_basics::parse(other_property_tokens.as_slice(), ::language::others::property_path)
            .expect("Parser result must be ok");
        assert_eq!(self.property.path, other_property.path);
        match other.1 {
            Some(other_expr) => {
                if let &UpdatingValue::Expression(ref expr) = &self.value {
                    expr.assert(other_expr)
                } else {
                    panic!("Pattern UpdatingValue::Expression not matches value {:?}", self.value);
                }
            },
            None => assert_eq!(self.value, UpdatingValue::Default),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Updating<'source> {
    pub low_priority: bool,
    pub ignore: bool,
    pub source: DataSourceAST<'source>,
    pub assignments: Vec<UpdatingAssignment<'source>>,
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
pub enum InsertingSource<'source> {
    ValueLists {
        properties: Option<Vec<ItemPath>>,
        lists: Vec<Vec<ExpressionAST<'source>>>,
    },
    AssignmentList {
        assignments: Vec<UpdatingAssignment<'source>>,
    },
    Selection {
        properties: Option<Vec<ItemPath>>,
        query: SelectionAST<'source>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Inserting<'source> {
    pub priority: InsertingPriority,
    pub ignore: bool,
    pub target: DataSourceAST<'source>,
    pub source: InsertingSource<'source>,
    pub on_duplicate_key_update: Option<Vec<UpdatingAssignment<'source>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deleting<'source> {
    pub low_priority: bool,
    pub quick: bool,
    pub ignore: bool,
    pub source: DataSourceAST<'source>,
    pub where_clause: Option<ExpressionAST<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItemAST<'source>>>,
    pub limit_clause: Option<u32>,
}
