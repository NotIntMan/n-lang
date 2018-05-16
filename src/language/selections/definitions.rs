//use helpers::IntoStatic;
use helpers::{
    Resolve,
    SyncRef,
};
use parser_basics::Identifier;
use language::{
    DataSource,
    DataSourceAST,
    DataType,
    Expression,
    ExpressionAST,
};
use project_analysis::{
    FunctionVariableScope,
    SemanticError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionResultSize {
    Usual,
    Small,
    Big,
    Buffered,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionExpressionAST<'source> {
    pub expr: ExpressionAST<'source>,
    pub alias: Option<Identifier<'source>>,
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for SelectionExpressionAST<'source> {
    type Result = SelectionExpression;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let expr = self.expr.resolve(scope)?;
        let alias = match &self.alias {
            Some(ident) => Some(ident.to_string()),
            None => None,
        };
        Ok(SelectionExpression {
            expr,
            alias,
        })
    }
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
pub struct SelectionExpression {
    // TODO Проверка на то, что все SelectionExpression имеют alias, либо все его не имеют для детерминированности определения выходного типа
    pub expr: Expression,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionResultAST<'source> {
    All,
    Some(Vec<SelectionExpressionAST<'source>>),
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

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for SelectionResultAST<'source> {
    type Result = SelectionResult;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let result = match self {
            SelectionResultAST::All => SelectionResult::All,
            SelectionResultAST::Some(vec) => SelectionResult::Some(
                vec.resolve(scope)?
            ),
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionResult {
    All,
    Some(Vec<SelectionExpression>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionSortingOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionSortingItemAST<'source> {
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

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for SelectionSortingItemAST<'source> {
    type Result = SelectionSortingItem;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let expr = self.expr.resolve(scope)?;
        Ok(SelectionSortingItem {
            expr,
            order: self.order,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionSortingItem {
    pub expr: Expression,
    pub order: SelectionSortingOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionGroupByClauseAST<'source> {
    pub sorting: Vec<SelectionSortingItemAST<'source>>,
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

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for SelectionGroupByClauseAST<'source> {
    type Result = SelectionGroupByClause;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let sorting = self.sorting.resolve(scope)?;
        Ok(SelectionGroupByClause {
            sorting,
            with_rollup: self.with_rollup,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionGroupByClause {
    pub sorting: Vec<SelectionSortingItem>,
    pub with_rollup: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionLimit {
    pub offset: Option<u32>,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionAST<'source> {
    pub distinct: bool,
    pub high_priority: bool,
    pub straight_join: bool,
    pub result_size: SelectionResultSize,
    pub cache: bool,
    pub result: SelectionResultAST<'source>,
    pub source: DataSourceAST<'source>,
    pub where_clause: Option<ExpressionAST<'source>>,
    pub group_by_clause: Option<SelectionGroupByClauseAST<'source>>,
    pub having_clause: Option<ExpressionAST<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItemAST<'source>>>,
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

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for SelectionAST<'source> {
    type Result = Selection;
    type Error = SemanticError;
    fn resolve(&self, _scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        //TODO Typeof selection
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    pub distinct: bool,
    pub high_priority: bool,
    pub straight_join: bool,
    pub result_size: SelectionResultSize,
    pub cache: bool,
    pub result: SelectionResult,
    pub source: DataSource,
    pub where_clause: Option<Expression>,
    pub group_by_clause: Option<SelectionGroupByClause>,
    pub having_clause: Option<Expression>,
    pub order_by_clause: Option<Vec<SelectionSortingItem>>,
    pub limit_clause: Option<SelectionLimit>,
    pub result_data_type: DataType,
}
