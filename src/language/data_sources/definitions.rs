//use helpers::IntoStatic;
use parser_basics::Identifier;
use language::{
    ExpressionAST,
    ItemPath,
    Selection,
};

#[derive(Debug, Clone, PartialEq)]
pub enum JoinCondition<'source> {
    Expression(ExpressionAST<'source>),
    Using(Vec<ItemPath>),
    Natural,
}

//impl<'source> IntoStatic for JoinCondition<'source> {
//    type Result = JoinCondition<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            JoinCondition::Expression(value) => JoinCondition::Expression(value.into_static()),
//            JoinCondition::Using(value) => JoinCondition::Using(value.into_static()),
//            JoinCondition::Natural => JoinCondition::Natural,
//        }
//    }
//}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Cross,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataSource<'source> {
    Table {
        name: ItemPath,
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

//impl<'source> IntoStatic for DataSource<'source> {
//    type Result = DataSource<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            DataSource::Table { name, alias } => DataSource::Table {
//                name: name.into_static(),
//                alias: alias.into_static(),
//            },
//            DataSource::Join { join_type, condition, left, right } => DataSource::Join {
//                join_type,
//                condition: condition.into_static(),
//                left: left.into_static(),
//                right: right.into_static(),
//            },
//            DataSource::Selection { query, alias } => DataSource::Selection {
//                query: query.into_static(),
//                alias: alias.into_static(),
//            },
//        }
//    }
//}
