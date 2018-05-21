use helpers::{
    Assertion,
    PathBuf,
    Resolve,
    SyncRef,
};
use lexeme_scanner::ItemPosition;
use language::{
    AssignmentTarget,
    DataSource,
    DataSourceAST,
    Expression,
    ExpressionAST,
    ItemPath,
    SelectionAST,
    SelectionSortingItem,
    SelectionSortingItemAST,
};
use project_analysis::{
    FunctionVariableScope,
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
    pub low_priority: bool,
    pub ignore: bool,
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
        if source.is_read_only() {
            return SemanticError::cannot_modify_readonly_datasource(self.pos)
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
            low_priority: self.low_priority,
            ignore: self.ignore,
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
    pub low_priority: bool,
    pub ignore: bool,
    pub source: DataSource,
    pub assignments: Vec<UpdatingAssignment>,
    pub where_clause: Option<Expression>,
    pub order_by_clause: Option<Vec<SelectionSortingItem>>,
    pub limit_clause: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertingPriority {
    Usual,
    Low,
    Delayed,
    High,
}

//TODO Typeof data source
#[derive(Debug, Clone, PartialEq)]
pub enum InsertingSourceAST<'source> {
    ValueLists {
        properties: Option<Vec<ItemPath>>,
        lists: Vec<Vec<ExpressionAST<'source>>>,
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
pub struct InsertingAST<'source> {
    pub priority: InsertingPriority,
    pub ignore: bool,
    pub target: DataSourceAST<'source>,
    pub source: InsertingSourceAST<'source>,
    pub on_duplicate_key_update: Option<Vec<UpdatingAssignmentAST<'source>>>,
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
