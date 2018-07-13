use helpers::{
    BlockFormatter,
    PathBuf,
    Resolve,
    SyncRef,
};
use indexmap::IndexMap;
use language::{
    BOOLEAN_TYPE,
    CompoundDataType,
    DataSource,
    DataSourceAST,
    DataType,
    Expression,
    ExpressionAST,
    Field,
    TSQLFunctionContext,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use project_analysis::{
    FunctionVariableScope,
    SemanticError,
};
use std::{
    fmt::{
        self,
        Write,
    },
    sync::Arc,
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

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionExpression {
    pub expr: Expression,
    pub alias: Option<String>,
}

impl SelectionExpression {
    pub fn can_be_named(&self) -> Option<String> {
        if let Some(name) = &self.alias {
            return Some(name.clone());
        }
        self.expr.can_be_named()
    }
    #[inline]
    pub fn make_field(&self) -> Field {
        Field {
            attributes: Vec::new(),
            field_type: self.expr.data_type.clone(),
        }
    }
    pub fn names_of_expression_set(expressions: &[SelectionExpression]) -> Option<Vec<String>> {
        if expressions.is_empty() {
            return None;
        }
        let mut expressions_iter = expressions.iter();
        let first_name = expressions_iter.next()?.can_be_named()?;
        let mut result = Vec::with_capacity(expressions.len());
        result.push(first_name);
        for expression in expressions_iter {
            match expression.can_be_named() {
                Some(name) => result.push(name),
                None => return None,
            }
        }
        Some(result)
    }
    pub fn type_of_expression_set(expressions: &[SelectionExpression]) -> DataType {
        match SelectionExpression::names_of_expression_set(expressions) {
            Some(names) => {
                let mut expressions_iter = expressions.iter();
                let mut fields = IndexMap::with_capacity(expressions.len());
                for name in names {
                    let expression = expressions_iter.next()
                        .expect("expression's set and set of theirs names should have equal sizes");
                    fields.insert(name, expression.make_field());
                }
                DataType::Compound(CompoundDataType::Structure(Arc::new(fields)))
            }
            None => {
                let mut fields = Vec::with_capacity(expressions.len());
                for expression in expressions {
                    fields.push(expression.make_field());
                }
                DataType::Compound(CompoundDataType::Tuple(Arc::new(fields)))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionResultAST<'source> {
    All(ItemPosition),
    Some(Vec<SelectionExpressionAST<'source>>),
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for SelectionResultAST<'source> {
    type Result = Vec<SelectionExpression>;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        match self {
            SelectionResultAST::All(pos) => {
                let scope_guard = scope.read();
                let mut results = Vec::new();
                let mut errors = Vec::new();
                let parent;
                let parent_guard;
                let variables = {
                    let parent_variables = match scope.parent() {
                        Some(parent_ref) => {
                            parent = parent_ref;
                            parent_guard = parent.read();
                            parent_guard.variables()
                        }
                        None => &[][..],
                    };
                    scope_guard.variables()
                        .iter()
                        .chain(parent_variables.iter())
                };
                for var in variables {
                    match var.data_type(*pos) {
                        Ok(data_type) => {
                            if errors.is_empty() {
                                results.push(SelectionExpression {
                                    expr: Expression::variable_access(var.clone(), *pos, data_type),
                                    alias: None,
                                });
                            }
                        }
                        Err(e) => {
                            errors.push(e);
                            continue;
                        }
                    }
                }
                if errors.is_empty() {
                    Ok(results)
                } else {
                    Err(errors)
                }
            }
            SelectionResultAST::Some(vec) => vec.resolve(scope),
        }
    }
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
    pub pos: ItemPosition,
}

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
    pub result: SelectionResultAST<'source>,
    pub source: DataSourceAST<'source>,
    pub where_clause: Option<ExpressionAST<'source>>,
    pub group_by_clause: Option<SelectionGroupByClauseAST<'source>>,
    pub having_clause: Option<ExpressionAST<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItemAST<'source>>>,
    pub limit_clause: Option<SelectionLimit>,
    pub pos: ItemPosition,
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for SelectionAST<'source> {
    type Result = Selection;
    type Error = SemanticError;
    fn resolve(&self, parent_scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        //TODO Non-array result of SELECT ... LIMIT 1 or aggregate SELECT without GROUP BY
        //TODO Moreover, result of SELECT ... LIMIT 1 must be nullable
        let scope = parent_scope.lite_weight_child();
        let aggregate_scope = scope.aggregate_child();

        let source = self.source.resolve(&scope)?;
        let result = self.result.resolve(&aggregate_scope)?;

        let mut errors = Vec::new();
        let is_aggregate_query = result.iter()
            .any(|expr| expr.expr.is_aggregate());

        let where_clause = match &self.where_clause {
            Some(where_clause) => where_clause.accumulative_resolve(&scope, &mut errors),
            None => None,
        };
        if let Some(where_clause) = &where_clause {
            if !where_clause.data_type.can_cast(&BOOLEAN_TYPE) {
                errors.push(SemanticError::expected_expression_of_another_type(
                    where_clause.pos,
                    BOOLEAN_TYPE.clone(),
                    where_clause.data_type.clone(),
                ));
            }
        }

        let order_by_clause = match &self.order_by_clause {
            Some(order_by_clause) => order_by_clause.accumulative_resolve(&scope, &mut errors),
            None => None,
        };

        let group_by_clause = match &self.group_by_clause {
            Some(group_by_clause) => {
                if is_aggregate_query {
                    group_by_clause.accumulative_resolve(&scope, &mut errors)
                } else {
                    errors.push(SemanticError::not_allowed_inside(
                        group_by_clause.pos,
                        "GROUP BY clause",
                        "not aggregate query",
                    ));
                    None
                }
            }
            None => None,
        };
        if is_aggregate_query {
            let items = match &group_by_clause {
                Some(clause) => clause.sorting.as_slice(),
                None => &[][..],
            };
            let aggregates = items.iter().map(|item| &item.expr);
            for expression in result.iter() {
                match expression.expr.can_be_selected_by_aggregation_query(aggregates.clone())
                    {
                        Ok(result) => {
                            if !result {
                                errors.push(SemanticError::not_allowed_inside(
                                    expression.expr.pos,
                                    "not aggregation expression",
                                    "aggregate query result",
                                ));
                            }
                        }
                        Err(mut local_errors) => {
                            errors.append(&mut local_errors);
                        }
                    }
            }
        }

        let having_clause = match &self.having_clause {
            Some(having_clause) => {
                if is_aggregate_query {
                    having_clause.accumulative_resolve(&aggregate_scope, &mut errors)
                } else {
                    errors.push(SemanticError::not_allowed_inside(
                        having_clause.pos,
                        "HAVING clause",
                        "not aggregate query",
                    ));
                    None
                }
            }
            None => None,
        };
        if let Some(having_clause) = &having_clause {
            if !having_clause.data_type.can_cast(&BOOLEAN_TYPE) {
                errors.push(SemanticError::expected_expression_of_another_type(
                    having_clause.pos,
                    BOOLEAN_TYPE.clone(),
                    having_clause.data_type.clone(),
                ));
            }
        }

        let result_data_type = SelectionExpression::type_of_expression_set(result.as_slice());
        let result_data_type = if is_aggregate_query && group_by_clause.is_none() {
            result_data_type
        } else {
            DataType::Array(Arc::new(result_data_type))
        };

        if let Some(limit_clause) = &self.limit_clause {
            if limit_clause.offset.is_some() {
                let is_order_by_clause_empty = match &order_by_clause {
                    Some(clause) => clause.is_empty(),
                    None => true,
                };
                if is_order_by_clause_empty {
                    errors.push(SemanticError::not_allowed_inside(
                        self.pos,
                        "LIMIT clause with offset",
                        "query without sorting",
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(Selection {
                distinct: self.distinct,
                result,
                source,
                where_clause,
                group_by_clause,
                having_clause,
                order_by_clause,
                limit_clause: self.limit_clause,
                result_data_type,
                pos: self.pos,
            })
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    pub distinct: bool,
    pub result: Vec<SelectionExpression>,
    pub source: DataSource,
    pub where_clause: Option<Expression>,
    pub group_by_clause: Option<SelectionGroupByClause>,
    pub having_clause: Option<Expression>,
    pub order_by_clause: Option<Vec<SelectionSortingItem>>,
    pub limit_clause: Option<SelectionLimit>,
    pub result_data_type: DataType,
    pub pos: ItemPosition,
}

impl Selection {
    pub fn fmt(
        &self,
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        let mut offset_fetch_clause = None;
        {
            let mut line = f.line()?;
            line.write_str("SELECT")?;
            if self.distinct {
                line.write_str(" DISTINCT")?;
            }
            if let Some(limit_clause) = &self.limit_clause {
                match &limit_clause.offset {
                    Some(offset) => {
                        offset_fetch_clause = Some((limit_clause.count, *offset));
                    }
                    None => {
                        write!(line, " TOP({})", limit_clause.count)?;
                    }
                }
            }
        }

        let mut sub_f = f.sub_block();

        for (i, result_item) in self.result.iter().enumerate() {
            let mut primitives = {
                let mut prefix = PathBuf::new(".");
                result_item.expr.data_type.primitives(prefix)
                    .into_iter()
                    .peekable()
            };

            while let Some(primitive) = primitives.next() {
                let mut line = sub_f.line()?;

                let new_path = primitive.path.as_path()
                    .into_new_buf("#");

                if let Some(sub_expr) = result_item.expr.get_property(primitive.path.as_path()) {
                    sub_expr.fmt(&mut line, context)?;
                } else {
                    write!(line, "( SELECT t.{} FROM (", new_path)?;
                    result_item.expr.fmt(&mut line, context)?;
                    write!(line, ") as t )")?;
                }

                let mut new_path = new_path;
                if let Some(alias) = result_item.can_be_named() {
                    new_path.push_back(alias);
                } else {
                    new_path.push_back(format_args!("component{}", i));
                }

                write!(line, " AS {}", new_path)?;

                if primitives.peek().is_some() {
                    write!(line, ",")?;
                }
            }
        }

        f.write_line("FROM")?;
        self.source.fmt(sub_f.clone(), context)?;

        if let Some(where_clause) = &self.where_clause {
            let mut line = f.line()?;
            line.write_str("WHERE ")?;
            where_clause.fmt(&mut line, context)?;
        }

        if let Some(group_by_clause) = &self.group_by_clause {
            let mut line = f.line()?;
            line.write_str("GROUP BY ")?;
            let mut items = group_by_clause.sorting.iter().peekable();
            while let Some(expr) = items.next() {
                expr.expr.fmt(&mut line, context)?;
                line.write_str(match &expr.order {
                    SelectionSortingOrder::Asc => " ASC",
                    SelectionSortingOrder::Desc => " DESC",
                })?;
                if items.peek().is_some() {
                    line.write_str(", ")?;
                }
            }
            if group_by_clause.with_rollup {
                line.write_str(" WITH ROLLUP")?;
            }
        }

        if let Some(having_clause) = &self.having_clause {
            let mut line = f.line()?;
            line.write_str("HAVING ")?;
            having_clause.fmt(&mut line, context)?;
        }

        {
            let order_by_clause = match &self.order_by_clause {
                Some(expressions) => &expressions[..],
                None => &[][..],
            };
            if !order_by_clause.is_empty() || offset_fetch_clause.is_some() {
                {
                    let mut line = f.line()?;
                    line.write_str("ORDER BY ")?;
                    if order_by_clause.is_empty() {
                        // TODO Проброс ошибок из генератора
                        unreachable!("LIMIT clause doesn't make sense without ORDER BY clause");
                    }
                    let mut items = order_by_clause.iter().peekable();
                    while let Some(expr) = items.next() {
                        expr.expr.fmt(&mut line, context)?;
                        line.write_str(match &expr.order {
                            SelectionSortingOrder::Asc => " ASC",
                            SelectionSortingOrder::Desc => " DESC",
                        })?;
                        if items.peek().is_some() {
                            line.write_str(", ")?;
                        }
                    }
                }
                if let Some((count, offset)) = offset_fetch_clause {
                    f.write_line(format_args!("OFFSET {} ROWS FETCH NEXT {} ROWS ONLY", offset, count))?;
                }
            }
        }

        Ok(())
    }
}
