use helpers::{
    accumulative_result_collect,
    Assertion,
    deep_result_collect,
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
};
use lexeme_scanner::ItemPosition;
use project_analysis::{
    FunctionVariableScope,
    InsertSourceContext,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertingPriority {
    Usual,
    Low,
    Delayed,
    High,
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
    AssignmentList {
        assignments: Vec<UpdatingAssignmentAST<'source>>,
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
            InsertingSourceASTBody::AssignmentList { assignments } => {
                let assignments = accumulative_result_collect(assignments.iter().map(|assignment_ast| {
                    let assignment = assignment_ast.resolve(ctx.scope)?;
                    if !ctx.target.is_target_belongs_to_source(&assignment.target) {
                        return SemanticError::not_allowed_inside(assignment_ast.pos, "assignment not belonging to the target data source", "INSERT query")
                            .into_err_vec();
                    }
                    Ok(assignment)
                }))?;
                Ok(InsertingSource::AssignmentList { assignments })
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
                            Some(assignments)
                        }
                        None => {
                            query_result_type.should_cast_to(query.pos, &ctx.target.get_datatype_for_insert(query.pos)?)?;
                            None
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
    AssignmentList {
        assignments: Vec<UpdatingAssignment>,
    },
    Selection {
        properties: Option<Vec<AssignmentTarget>>,
        query: Selection,
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

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for InsertingAST<'source> {
    type Result = Inserting;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let target = self.target.resolve(scope)?;
        let mut errors = Vec::new();

        let source = {
            let ctx = InsertSourceContext {
                scope,
                target: &target,
            };
            self.source.accumulative_resolve(&ctx, &mut errors)
        };
        let on_duplicate_key_update = self.on_duplicate_key_update.accumulative_resolve(scope, &mut errors);

        let source = match source {
            Some(x) => x,
            None => return Err(errors)
        };
        let on_duplicate_key_update = match on_duplicate_key_update {
            Some(x) => x,
            None => return Err(errors)
        };

        Ok(Inserting {
            priority: self.priority,
            ignore: self.ignore,
            target,
            source,
            on_duplicate_key_update,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Inserting {
    pub priority: InsertingPriority,
    pub ignore: bool,
    pub target: DataSource,
    pub source: InsertingSource,
    pub on_duplicate_key_update: Option<Vec<UpdatingAssignment>>,
}

impl Inserting {
    #[inline]
    pub fn is_lite_weight(&self) -> bool {
        self.target.is_local()
    }
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
            low_priority: self.low_priority,
            quick: self.quick,
            ignore: self.ignore,
            source,
            where_clause,
            order_by_clause,
            limit_clause: self.limit_clause,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deleting {
    pub low_priority: bool,
    pub quick: bool,
    pub ignore: bool,
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
}
