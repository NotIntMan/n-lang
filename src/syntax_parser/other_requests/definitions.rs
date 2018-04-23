use helpers::assertion::Assertion;
//use helpers::into_static::IntoStatic;
use syntax_parser::expressions::Expression;
use syntax_parser::data_sources::DataSource;
use syntax_parser::selections::{
    Selection,
    SelectionSortingItem,
};
use syntax_parser::others::ItemPath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatingValue<'source> {
    Default,
    Expression(Expression<'source>),
}

//impl<'source> IntoStatic for UpdatingValue<'source> {
//    type Result = UpdatingValue<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            UpdatingValue::Default => UpdatingValue::Default,
//            UpdatingValue::Expression(value) => UpdatingValue::Expression(value.into_static()),
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatingAssignment<'source> {
    pub property: ItemPath,
    pub value: UpdatingValue<'source>,
}

//impl<'source> IntoStatic for UpdatingAssignment<'source> {
//    type Result = UpdatingAssignment<'static>;
//    fn into_static(self) -> Self::Result {
//        let UpdatingAssignment {
//            property,
//            value,
//        } = self;
//        UpdatingAssignment {
//            property: property.into_static(),
//            value: value.into_static(),
//        }
//    }
//}

impl<'a, 'b, 'source> Assertion<(&'a str, Option<&'b str>)> for UpdatingAssignment<'source> {
    fn assert(&self, other: &(&str, Option<&str>)) {
        let other_property_tokens = ::lexeme_scanner::Scanner::scan(other.0)
            .expect("Scanner result must be ok");
        let other_property = ::parser_basics::parse(other_property_tokens.as_slice(), ::syntax_parser::others::property_path)
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

//impl<'source> IntoStatic for Updating<'source> {
//    type Result = Updating<'static>;
//    fn into_static(self) -> Self::Result {
//        let Updating {
//            low_priority,
//            ignore,
//            source,
//            assignments,
//            where_clause,
//            order_by_clause,
//            limit_clause,
//        } = self;
//        Updating {
//            low_priority,
//            ignore,
//            source: source.into_static(),
//            assignments: assignments.into_static(),
//            where_clause: where_clause.into_static(),
//            order_by_clause: order_by_clause.into_static(),
//            limit_clause,
//        }
//    }
//}

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
        properties: Option<Vec<ItemPath>>,
        lists: Vec<Vec<Expression<'source>>>,
    },
    AssignmentList {
        assignments: Vec<UpdatingAssignment<'source>>,
    },
    Selection {
        properties: Option<Vec<ItemPath>>,
        query: Selection<'source>,
    },
}

//impl<'source> IntoStatic for InsertingSource<'source> {
//    type Result = InsertingSource<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            InsertingSource::ValueLists { properties, lists } => InsertingSource::ValueLists {
//                properties: properties.into_static(),
//                lists: lists.into_static(),
//            },
//            InsertingSource::AssignmentList { assignments } => InsertingSource::AssignmentList {
//                assignments: assignments.into_static(),
//            },
//            InsertingSource::Selection { properties, query } => InsertingSource::Selection {
//                properties: properties.into_static(),
//                query: query.into_static(),
//            },
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inserting<'source> {
    pub priority: InsertingPriority,
    pub ignore: bool,
    pub target: DataSource<'source>,
    pub source: InsertingSource<'source>,
    pub on_duplicate_key_update: Option<Vec<UpdatingAssignment<'source>>>,
}

//impl<'source> IntoStatic for Inserting<'source> {
//    type Result = Inserting<'static>;
//    fn into_static(self) -> Self::Result {
//        let Inserting {
//            priority,
//            ignore,
//            target,
//            source,
//            on_duplicate_key_update,
//        } = self;
//        Inserting {
//            priority,
//            ignore,
//            target: target.into_static(),
//            source: source.into_static(),
//            on_duplicate_key_update: on_duplicate_key_update.into_static(),
//        }
//    }
//}

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

//impl<'source> IntoStatic for Deleting<'source> {
//    type Result = Deleting<'static>;
//    fn into_static(self) -> Self::Result {
//        let Deleting {
//            low_priority,
//            quick,
//            ignore,
//            source,
//            where_clause,
//            order_by_clause,
//            limit_clause,
//        } = self;
//        Deleting {
//            low_priority,
//            quick,
//            ignore,
//            source: source.into_static(),
//            where_clause: where_clause.into_static(),
//            order_by_clause: order_by_clause.into_static(),
//            limit_clause,
//        }
//    }
//}
