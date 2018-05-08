//use helpers::IntoStatic;
use parser_basics::Identifier;
use language::{
    DataSource,
    ExpressionAST,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionResultSize {
    Usual,
    Small,
    Big,
    Buffered,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionExpression<'source> {
    pub expr: ExpressionAST<'source>,
    pub alias: Option<Identifier<'source>>,
}

//impl<'source> IntoStatic for SelectionExpression<'source> {
//    type Result = SelectionExpression<'static>;
//    fn into_static(self) -> Self::Result {
//        let SelectionExpression { expr, alias } = self;
//        SelectionExpression {
//            expr: expr.into_static(),
//            alias: alias.into_static(),
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionResult<'source> {
    All,
    Some(Vec<SelectionExpression<'source>>),
}

//impl<'source> IntoStatic for SelectionResult<'source> {
//    type Result = SelectionResult<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            SelectionResult::All => SelectionResult::All,
//            SelectionResult::Some(expressions) => SelectionResult::Some(expressions.into_static()),
//        }
//    }
//}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionSortingOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionSortingItem<'source> {
    pub expr: ExpressionAST<'source>,
    pub order: SelectionSortingOrder,
}

//impl<'source> IntoStatic for SelectionSortingItem<'source> {
//    type Result = SelectionSortingItem<'static>;
//    fn into_static(self) -> Self::Result {
//        let SelectionSortingItem { expr, order } = self;
//        SelectionSortingItem {
//            expr: expr.into_static(),
//            order,
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionGroupByClause<'source> {
    pub sorting: Vec<SelectionSortingItem<'source>>,
    pub with_rollup: bool,
}

//impl<'source> IntoStatic for SelectionGroupByClause<'source> {
//    type Result = SelectionGroupByClause<'static>;
//    fn into_static(self) -> Self::Result {
//        let SelectionGroupByClause { sorting, with_rollup } = self;
//        SelectionGroupByClause {
//            sorting: sorting.into_static(),
//            with_rollup,
//        }
//    }
//}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionLimit {
    pub offset: Option<u32>,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selection<'source> {
    pub distinct: bool,
    pub high_priority: bool,
    pub straight_join: bool,
    pub result_size: SelectionResultSize,
    pub cache: bool,
    pub result: SelectionResult<'source>,
    pub source: DataSource<'source>,
    pub where_clause: Option<ExpressionAST<'source>>,
    pub group_by_clause: Option<SelectionGroupByClause<'source>>,
    pub having_clause: Option<ExpressionAST<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItem<'source>>>,
    pub limit_clause: Option<SelectionLimit>,
}

//impl<'source> IntoStatic for Selection<'source> {
//    type Result = Selection<'static>;
//    fn into_static(self) -> Self::Result {
//        let Selection {
//            distinct,
//            high_priority,
//            straight_join,
//            result_size,
//            cache,
//            result,
//            source,
//            where_clause,
//            group_by_clause,
//            having_clause,
//            order_by_clause,
//            limit_clause,
//        } = self;
//        Selection {
//            distinct,
//            high_priority,
//            straight_join,
//            result_size,
//            cache,
//            result: result.into_static(),
//            source: source.into_static(),
//            where_clause: where_clause.into_static(),
//            group_by_clause: group_by_clause.into_static(),
//            having_clause: having_clause.into_static(),
//            order_by_clause: order_by_clause.into_static(),
//            limit_clause,
//        }
//    }
//}
