use helpers::{
    accumulative_result_collect,
    Assertion,
    BlockFormatter,
    deep_result_collect,
    PathBuf,
    Resolve,
    SyncRef,
};
use language::{
    AssignmentTarget,
    DataSource,
    DataSourceAST,
    DataType,
    Expression,
    ExpressionAST,
    ItemPath,
    Selection,
    SelectionAST,
    SelectionSortingItem,
    SelectionSortingItemAST,
    SelectionSortingOrder,
    TSQLFunctionContext,
};
use lexeme_scanner::ItemPosition;
use project_analysis::{
    FunctionVariableScope,
    InsertSourceContext,
    SemanticError,
};
use std::fmt::{
    self,
    Write,
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
    pub pos: ItemPosition,
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for UpdatingAssignmentAST<'source> {
    type Result = UpdatingAssignment;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let value = self.value.resolve(scope)?;
        let target = AssignmentTarget::new_in_scope(
            scope,
            self.property.pos,
            self.property.path.as_path(),
        )?;
        target.check_source_type(&value.data_type)?;
        Ok(UpdatingAssignment {
            target,
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatingAssignment {
    pub target: AssignmentTarget,
    pub value: Expression,
}

impl UpdatingAssignment {
    pub fn fmt(
        &self,
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
        last_comma: bool,
    ) -> fmt::Result {
        let var_guard = self.target.var.read();
        let var_data_type = var_guard.data_type()
            .expect("Variable cannot have unknown data-type at generate time.")
            .property_type(self.target.pos, self.target.property.as_path())
            .expect("Property existing should be already checked at generate time.");

        if var_data_type.as_primitive().is_some() {
            let mut line = f.line()?;
            if !var_guard.is_automatic() {
                line.write_char('@')?;
            }
            line.write_str(var_guard.name())?;
            if !self.target.property.is_empty() {
                write!(line, ".{}", self.target.property.as_path().into_new_buf("#"))?;
            }
            line.write_str(" = ")?;
            self.value.fmt(&mut line, context)?;
            if last_comma {
                line.write_char(',')?;
            }
        } else {
            let mut primitives = var_data_type.primitives(PathBuf::new("#"))
                .into_iter()
                .peekable();

            while let Some(primitive) = primitives.next() {
                let mut line = f.line()?;
                if !var_guard.is_automatic() {
                    line.write_char('@')?;
                }
                line.write_str(var_guard.name())?;

                let mut target_path = self.target.property.as_path().into_new_buf("#");
                target_path.append(primitive.path.as_path());

                write!(line, ".{} = ", target_path)?;
                Expression::fmt_property_access(
                    &mut line,
                    &self.value,
                    primitive.path.as_path(),
                    context,
                )?;
                if last_comma || primitives.peek().is_some() {
                    line.write_char(',')?;
                }
            }
        }
        Ok(())
    }
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
            }
            None => match_it!(&self.value, UpdatingValueAST::Default(_) => {}),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdatingAST<'source> {
    pub source: DataSourceAST<'source>,
    pub assignments: Vec<UpdatingAssignmentAST<'source>>,
    pub where_clause: Option<ExpressionAST<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItemAST<'source>>>,
    pub limit_clause: Option<u32>,
    pub pos: ItemPosition,
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for UpdatingAST<'source> {
    type Result = Updating;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let source = self.source.resolve(scope)?;
        if !source.is_allows_updates() {
            return SemanticError::cannot_do_with_datasource(self.pos, "update")
                .into_err_vec();
        }
        let mut errors = Vec::new();

        let assignments = self.assignments.accumulative_resolve(scope, &mut errors);
        let where_clause = self.where_clause.accumulative_resolve(scope, &mut errors);
        let order_by_clause = self.order_by_clause.accumulative_resolve(scope, &mut errors);

        let assignments = match assignments {
            Some(x) => x,
            None => return Err(errors),
        };
        let where_clause = match where_clause {
            Some(x) => x,
            None => return Err(errors),
        };
        let order_by_clause = match order_by_clause {
            Some(x) => x,
            None => return Err(errors),
        };

        Ok(Updating {
            source,
            assignments,
            where_clause,
            order_by_clause,
            limit_clause: self.limit_clause,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Updating {
    pub source: DataSource,
    pub assignments: Vec<UpdatingAssignment>,
    pub where_clause: Option<Expression>,
    pub order_by_clause: Option<Vec<SelectionSortingItem>>,
    pub limit_clause: Option<u32>,
}

impl Updating {
    #[inline]
    pub fn is_lite_weight(&self) -> bool {
        self.source.is_local()
    }
    pub fn fmt(
        &self,
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        f.write_line("UPDATE")?;
        let mut sub_f = f.sub_block();
        let sub_sub_f = sub_f.sub_block();
        if let Some(limit) = &self.limit_clause {
            sub_f.write_line(format_args!("TOP({})", limit))?;
        }
        self.source.fmt(sub_sub_f.clone(), context)?;
        sub_f.write_line("SET")?;
        {
            let mut assignments = self.assignments.iter()
                .peekable();
            while let Some(assignment) = assignments.next() {
                assignment.fmt(
                    sub_sub_f.clone(),
                    context,
                    assignments.peek().is_some(),
                )?;
            }
        }
        if let Some(where_clause) = &self.where_clause {
            let mut line = sub_f.line()?;
            line.write_str("WHERE ")?;
            where_clause.fmt(&mut line, context)?;
        }
        if let Some(order_by_clause) = &self.order_by_clause {
            let mut line = sub_f.line()?;
            line.write_str("ORDER BY ")?;
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
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueList<'source> {
    pub values: Vec<ExpressionAST<'source>>,
    pub pos: ItemPosition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InsertingSourceASTBody<'source> {
    ValueLists {
        properties: Option<Vec<ItemPath>>,
        lists: Vec<ValueList<'source>>,
    },
    Selection {
        properties: Option<Vec<ItemPath>>,
        query: SelectionAST<'source>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertingSourceAST<'source> {
    pub body: InsertingSourceASTBody<'source>,
    pub pos: ItemPosition,
}

impl<'source, 'a> Resolve<InsertSourceContext<'a>> for InsertingSourceAST<'source> {
    type Result = InsertingSource;
    type Error = SemanticError;
    fn resolve(&self, ctx: &InsertSourceContext<'a>) -> Result<Self::Result, Vec<Self::Error>> {
        match &self.body {
            InsertingSourceASTBody::ValueLists { properties, lists } => {
                let properties = match properties {
                    Some(properties) => properties,
                    None => return SemanticError::not_supported_yet(self.pos, "lists of values without list of columns as a data source")
                        .into_err_vec(),
                };
                let properties: Vec<AssignmentTarget> = deep_result_collect(
                    properties.iter()
                        .map(|prop| {
                            let assignment = AssignmentTarget::new_in_scope(ctx.scope, prop.pos, prop.path.as_path())?;
                            if !ctx.target.is_target_belongs_to_source(&assignment) {
                                return Err(SemanticError::not_allowed_inside(prop.pos, "assignment not belonging to the target data source", "INSERT query"));
                            }
                            Ok(assignment)
                        })
                )?;
                let expected_len = properties.len();
                let lists: Vec<Vec<Expression>> = accumulative_result_collect(lists.iter().map(|list| {
                    let mut errors = Vec::new();
                    let got_len = list.values.len();
                    if got_len != expected_len {
                        errors.push(SemanticError::value_list_with_wrong_length(list.pos, expected_len, got_len));
                    }
                    let expressions = match list.values.accumulative_resolve(ctx.scope, &mut errors) {
                        Some(expressions) => expressions,
                        None => return Err(errors),
                    };
                    for (i, prop) in properties.iter().enumerate() {
                        if let Err(e) = prop.check_source_type(&expressions[i].data_type) {
                            errors.push(e);
                        }
                    }
                    if errors.is_empty() {
                        Ok(expressions)
                    } else {
                        Err(errors)
                    }
                }))?;
                Ok(InsertingSource::ValueLists {
                    properties,
                    lists,
                })
            }
            InsertingSourceASTBody::Selection { properties, query } => {
                let query = query.resolve(ctx.scope)?;
                let properties = {
                    let query_result_type: &DataType = match &query.result_data_type {
                        DataType::Array(query_result_type) => &**query_result_type,
                        query_result_type => query_result_type,
                    };
                    match properties {
                        Some(properties) => {
                            let assignments: Vec<AssignmentTarget> = accumulative_result_collect(
                                properties.iter().enumerate()
                                    .map(|(i, prop)| {
                                        let assignment = AssignmentTarget::new_in_scope(
                                            ctx.scope,
                                            prop.pos,
                                            prop.path.as_path(),
                                        )?;
                                        let query_field_type = match query_result_type.get_field_type(i) {
                                            Some(field_type) => field_type,
                                            None => return SemanticError::select_with_wrong_column_count(query.pos, properties.len(), query.result_data_type.field_len())
                                                .into_err_vec(),
                                        };
                                        assignment.check_source_type(&query_field_type)?;
                                        Ok(assignment)
                                    })
                            )?;
                            assignments
                        }
                        None => {
                            let var = ctx.target.get_target_for_insert(query.pos)?;
                            let target_data_type = var.data_type(query.pos)?;
                            query_result_type.should_cast_to(query.pos, &target_data_type)?;
                            let mut primitives_prefix = PathBuf::new(".");
                            primitives_prefix.push(var.read().name());
                            let properties: Result<_, _> = target_data_type.primitives(primitives_prefix)
                                .into_iter()
                                .map(|primitive| AssignmentTarget::new_in_scope(
                                    ctx.scope,
                                    query.pos,
                                    primitive.path.as_path(),
                                ))
                                .collect();
                            properties?
                        }
                    }
                };
                Ok(InsertingSource::Selection { properties, query })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InsertingSource {
    ValueLists {
        properties: Vec<AssignmentTarget>,
        lists: Vec<Vec<Expression>>,
    },
    Selection {
        properties: Vec<AssignmentTarget>,
        query: Selection,
    },
}

impl InsertingSource {
    pub fn fmt_target_list(
        target: &[AssignmentTarget],
        line: &mut impl fmt::Write,
    ) -> fmt::Result {
        line.write_char('(')?;

        let mut properties = target.iter()
            .peekable();
        while let Some(property) = properties.next() {
            let var_guard = property.var.read();
            let mut primitives = var_guard.data_type()
                .expect("Variable data-type should be known at generate time")
                .property_type(ItemPosition::default(), property.property.as_path())
                .expect("Property existing should be already checked at generate time.")
                .primitives(property.property.as_path().into_new_buf("#"))
                .into_iter()
                .peekable();
            while let Some(primitive) = primitives.next() {
                if !var_guard.is_automatic() {
                    line.write_char('@')?;
                }
                write!(line, "{}.{}", var_guard.name(), primitive.path)?;
                if primitives.peek().is_some() || properties.peek().is_some() {
                    line.write_str(", ")?;
                }
            }
        }

        line.write_char(')')
    }
    pub fn fmt(
        &self,
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        let mut sub_f = f.sub_block();
        match self {
            InsertingSource::ValueLists { properties, lists } => {
                InsertingSource::fmt_target_list(&properties, &mut f.line()?)?;
                f.write_line("VALUES")?;
                let mut lists_iter = lists.iter()
                    .peekable();
                while let Some(list) = lists_iter.next() {
                    f.write_line("(")?;
                    let mut expressions = list.iter()
                        .enumerate()
                        .peekable();
                    while let Some((i, expression)) = expressions.next() {
                        let property = &properties[i];
                        let var_guard = property.var.read();
                        let mut primitives = var_guard.data_type()
                            .expect("Variable data-type should be known at generate time")
                            .property_type(ItemPosition::default(), property.property.as_path())
                            .expect("Property existing should be already checked at generate time.")
                            .primitives(PathBuf::new("#"))
                            .into_iter()
                            .peekable();
                        while let Some(primitive) = primitives.next() {
                            let mut line = sub_f.line()?;
                            Expression::fmt_property_access(
                                &mut line,
                                expression,
                                primitive.path.as_path(),
                                context,
                            )?;
                            if primitives.peek().is_some() || expressions.peek().is_some() {
                                line.write_char(',')?;
                            }
                        }
                    }
                    if lists_iter.peek().is_some() {
                        f.write_line("),")?;
                    } else {
                        f.write_line(")")?;
                    }
                }
                Ok(())
            }
            InsertingSource::Selection { properties, query } => {
                f.write_line("SELECT")?;
                let mut properties_iter = properties.iter()
                    .peekable();
                while let Some(property) = properties_iter.next() {
                    let var_guard = property.var.read();
                    let mut primitives = var_guard.data_type()
                        .expect("Variable data-type should be known at generate time")
                        .property_type(ItemPosition::default(), property.property.as_path())
                        .expect("Property existing should be already checked at generate time.")
                        .primitives(property.property.as_path().into_new_buf("#"))
                        .into_iter()
                        .peekable();
                    while let Some(primitive) = primitives.next() {
                        let mut line = sub_f.line()?;
                        write!(line, "t.{path} as {path}", path = primitive.path)?;
                        if primitives.peek().is_some() || properties_iter.peek().is_some() {
                            line.write_char(',')?;
                        }
                    }
                }
                f.write_line("FROM (")?;
                query.fmt(sub_f, context)?;
                f.write_line(") as t")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertingAST<'source> {
    pub target: DataSourceAST<'source>,
    pub source: InsertingSourceAST<'source>,
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for InsertingAST<'source> {
    type Result = Inserting;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let target = self.target.resolve(scope)?;

        let source = {
            let ctx = InsertSourceContext {
                scope,
                target: &target,
            };
            self.source.resolve(&ctx)?
        };

        Ok(Inserting {
            target,
            source,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Inserting {
    pub target: DataSource,
    pub source: InsertingSource,
}

impl Inserting {
    #[inline]
    pub fn is_lite_weight(&self) -> bool {
        self.target.is_local()
    }
    pub fn fmt(
        &self,
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        f.write_line("INSERT INTO")?;
        let sub_f = f.sub_block();
        self.target.fmt(sub_f.clone(), context)?;
        self.source.fmt(sub_f, context)?;
        f.write_line(";")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeletingAST<'source> {
    pub source: DataSourceAST<'source>,
    pub where_clause: Option<ExpressionAST<'source>>,
    pub order_by_clause: Option<Vec<SelectionSortingItemAST<'source>>>,
    pub limit_clause: Option<u32>,
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for DeletingAST<'source> {
    type Result = Deleting;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let source = self.source.resolve(scope)?;
        let mut errors = Vec::new();

        let where_clause = self.where_clause.accumulative_resolve(scope, &mut errors);
        let order_by_clause = self.order_by_clause.accumulative_resolve(scope, &mut errors);

        let where_clause = match where_clause {
            Some(x) => x,
            None => return Err(errors)
        };
        let order_by_clause = match order_by_clause {
            Some(x) => x,
            None => return Err(errors)
        };

        Ok(Deleting {
            source,
            where_clause,
            order_by_clause,
            limit_clause: self.limit_clause,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deleting {
    pub source: DataSource,
    pub where_clause: Option<Expression>,
    pub order_by_clause: Option<Vec<SelectionSortingItem>>,
    pub limit_clause: Option<u32>,
}

impl Deleting {
    #[inline]
    pub fn is_lite_weight(&self) -> bool {
        self.source.is_local()
    }
    pub fn fmt(
        &self,
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        match &self.limit_clause {
            Some(limit) => f.write_line(format_args!("DELETE TOP({}) FROM", limit))?,
            None => f.write_line("DELETE FROM")?,
        }
        let mut sub_f = f.sub_block();
        self.source.fmt(sub_f.sub_block(), context)?;
        if let Some(where_clause) = &self.where_clause {
            let mut line = sub_f.line()?;
            line.write_str("WHERE ")?;
            where_clause.fmt(&mut line, context)?;
        }
        sub_f.write_line(';')
    }
}
