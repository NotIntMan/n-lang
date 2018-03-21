use helpers::assertion::Assertion;
use syntax_parser::expressions::Expression;
use syntax_parser::data_sources::DataSource;
use syntax_parser::selections::{
    Selection,
    SelectionSortingItem,
};
use syntax_parser::others::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatingValue<'source> {
    Default,
    Expression(Expression<'source>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatingAssignment<'source> {
    pub property: Path<'source>,
    pub value: UpdatingValue<'source>,
}

impl<'a, 'b, 'source> Assertion<(&'a str, Option<&'b str>)> for UpdatingAssignment<'source> {
    fn assert(&self, other: &(&str, Option<&str>)) {
        let other_property_tokens = ::lexeme_scanner::Scanner::scan(other.0)
            .expect("Scanner result must be ok");
        let other_property = ::parser_basics::parse(other_property_tokens.as_slice(), ::syntax_parser::others::property_path)
            .expect("Parser result must be ok");
        assert_eq!(self.property, other_property);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Updating<'source> {
    pub low_priority: bool,
    pub ignore: bool,
    pub source: DataSource<'source>,
    pub assignments: Vec<UpdatingAssignment<'source>>,
    pub where_clause: Option<Expression<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItem<'source>>>,
    pub limit_clause: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertingPriority {
    Usual,
    Low,
    Delayed,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsertingSource<'source> {
    ValueLists {
        properties: Option<Vec<Path<'source>>>,
        lists: Vec<Vec<Expression<'source>>>,
    },
    AssignmentList {
        assignments: Vec<UpdatingAssignment<'source>>,
    },
    Selection {
        properties: Option<Vec<Path<'source>>>,
        query: Selection<'source>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inserting<'source> {
    pub priority: InsertingPriority,
    pub ignore: bool,
    pub target: DataSource<'source>,
    pub source: InsertingSource<'source>,
    pub on_duplicate_key_update: Option<Vec<UpdatingAssignment<'source>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deleting<'source> {
    pub low_priority: bool,
    pub quick: bool,
    pub ignore: bool,
    pub source: DataSource<'source>,
    pub where_clause: Option<Expression<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItem<'source>>>,
    pub limit_clause: Option<u32>,
}
