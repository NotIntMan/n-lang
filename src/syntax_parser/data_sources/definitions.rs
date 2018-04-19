//use helpers::into_static::IntoStatic;
use parser_basics::Identifier;
use syntax_parser::expressions::Expression;
use syntax_parser::selections::Selection;
use syntax_parser::others::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinCondition<'source> {
    Expression(Expression<'source>),
    Using(Vec<Path<'source>>),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSource<'source> {
    Table {
        name: Path<'source>,
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
