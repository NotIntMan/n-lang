use helpers::{
    BlockFormatter,
    Path,
    Resolve,
    SyncRef,
};
use language::{
    AssignmentTarget,
    DataType,
    Expression,
    ExpressionAST,
    ItemPath,
    Selection,
    SelectionAST,
    TSQLFunctionContext,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use project_analysis::{
    FunctionVariable,
    FunctionVariableScope,
    Item,
    SemanticError,
    SemanticItemType,
};
use std::fmt::{
    self,
    Write,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Cross,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataSourceAST<'source> {
    Table {
        name: ItemPath,
        alias: Option<Identifier<'source>>,
    },
    Join {
        join_type: JoinType,
        condition: Option<ExpressionAST<'source>>,
        left: Box<DataSourceAST<'source>>,
        right: Box<DataSourceAST<'source>>,
    },
    Selection {
        query: Box<SelectionAST<'source>>,
        alias: Identifier<'source>,
    },
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for DataSourceAST<'source> {
    type Result = DataSource;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        match self {
            DataSourceAST::Table { name, alias } => {
                let pos = name.pos;
                if let Some(name) = name.path.as_path().the_only() {
                    if let Some(var) = scope.get_variable(name) {
                        let var_data_type = var.property_type(pos, Path::empty())?;
                        let entity_type = match var_data_type {
                            DataType::Array(entity_type) => (*entity_type).clone(),
                            _ => return SemanticError::not_allowed_here(pos, "not array variable").into_err_vec(),
                        };
                        let new_var_name = match alias {
                            Some(alias) => alias.text(),
                            None => name,
                        };
                        let new_var = scope.new_variable(pos, new_var_name.to_string(), Some(entity_type))?;
                        if var.is_read_only() {
                            new_var.make_read_only();
                        }
                        new_var.mark_as_automatic();
                        return Ok(DataSource::Variable { var });
                    }
                }
                match scope.module().get_item(name.path.as_path(), &mut Vec::new()) {
                    Some(item) => {
                        let var = {
                            let mut item = item.read();
                            let item_type = item.get_type();
                            let entity_type = match item.get_table() {
                                Some(table) => &table.entity,
                                None => return SemanticError::expected_item_of_another_type(
                                    name.pos,
                                    SemanticItemType::Table,
                                    item_type,
                                )
                                    .into_err_vec(),
                            };
                            let new_var_name = match alias {
                                Some(alias) => alias.text(),
                                None => name.path.as_path()
                                    .pop_right()
                                    .expect("Item's path should not be null"),
                            };
                            let var = scope.new_variable(pos, new_var_name.to_string(), Some(entity_type.clone()))?;
                            {
                                let mut var_guard = var.write();
                                var_guard.mark_as_automatic();
                            }
                            var
                        };
                        Ok(DataSource::Table { item, var })
                    }
                    None => SemanticError::unresolved_item(name.pos, name.path.clone()).into_err_vec(),
                }
            }
            DataSourceAST::Join { join_type, condition, left, right } => {
                let left = left.resolve(scope)?;
                let right = right.resolve(scope)?;
                let condition = condition.resolve(scope)?;
                Ok(DataSource::Join { join_type: *join_type, condition, left, right })
            }
            DataSourceAST::Selection { query, alias } => {
                let scope = scope.parent()
                    .expect("Sub-selection cannot resolve on scope without parent (because of isolated scope reasons)");
                let query: Box<Selection> = query.resolve(&scope)?;
                let var = scope.new_variable(
                    alias.item_pos(),
                    alias.to_string(),
                    Some(query.result_data_type.clone()),
                )?;
                Ok(DataSource::Selection { query, alias: alias.to_string(), var })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataSource {
    Variable {
        var: SyncRef<FunctionVariable>,
    },
    Table {
        item: SyncRef<Item>,
        var: SyncRef<FunctionVariable>,
    },
    Join {
        join_type: JoinType,
        condition: Option<Expression>,
        left: Box<DataSource>,
        right: Box<DataSource>,
    },
    Selection {
        query: Box<Selection>,
        alias: String,
        var: SyncRef<FunctionVariable>,
    },
}

impl DataSource {
    pub fn is_allows_updates(&self) -> bool {
        match self {
            DataSource::Variable { var } => !var.is_read_only(),
            DataSource::Table { item: _, var: _ } => true,
            DataSource::Join { join_type: _, condition: _, left, right } => left.is_allows_updates() && right.is_allows_updates(),
            DataSource::Selection { query: _, alias: _, var: _ } => false,
        }
    }
    pub fn is_allows_inserts(&self) -> bool {
        match self {
            DataSource::Variable { var } => !var.is_read_only(),
            DataSource::Table { item: _, var: _ } => true,
            DataSource::Join { join_type: _, condition: _, left: _, right: _ } => false,
            DataSource::Selection { query: _, alias: _, var: _ } => false,
        }
    }
    pub fn get_target_for_insert(&self, pos: ItemPosition) -> Result<SyncRef<FunctionVariable>, SemanticError> {
        match self {
            DataSource::Variable { var } => Ok(var.clone()),
            DataSource::Table { item: _, var } => Ok(var.clone()),
            DataSource::Join { join_type: _, condition: _, left: _, right: _ } =>
                return Err(SemanticError::not_allowed_inside(pos, "insertion", "JOIN of data sources")),
            DataSource::Selection { query: _, alias: _, var: _ } =>
                return Err(SemanticError::not_allowed_inside(pos, "insertion", "SELECT subquery")),
        }
    }
    pub fn is_allows_deletes(&self) -> bool {
        match self {
            DataSource::Variable { var } => !var.is_read_only(),
            DataSource::Table { item: _, var: _ } => true,
            DataSource::Join { join_type: _, condition: _, left: _, right: _ } => false,
            DataSource::Selection { query: _, alias: _, var: _ } => false,
        }
    }
    pub fn is_target_belongs_to_source(&self, target: &AssignmentTarget) -> bool {
        let inner_var = match self {
            DataSource::Variable { var } => var,
            DataSource::Table { item: _, var } => var,
            DataSource::Join { join_type: _, condition: _, left, right } =>
                return left.is_target_belongs_to_source(target) || right.is_target_belongs_to_source(target),
            DataSource::Selection { query: _, alias: _, var } => var,
        };
        inner_var.is_same_ref(&target.var)
    }
    pub fn is_local(&self) -> bool {
        match self {
            DataSource::Variable { var: _ } => true,
            DataSource::Table { item: _, var: _ } => false,
            DataSource::Join { join_type: _, condition: _, left, right } =>
                left.is_local() && right.is_local(),
            DataSource::Selection { query, alias: _, var: _ } => query.source.is_local(),
        }
    }
    pub fn fmt(
        &self,
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
        aliases: bool,
    ) -> fmt::Result {
        match self {
            DataSource::Variable { var } => {
                let var_guard = var.read();
                if aliases {
                    f.write_line(format_args!("@{name} AS {name}", name = var_guard.name()))
                } else {
                    f.write_line(format_args!("@{name}", name = var_guard.name()))
                }
            }
            DataSource::Table { item, var } => {
                let item_guard = item.read();
                if aliases {
                    let var_guard = var.read();
                    f.write_line(format_args!("[{}] AS {}", item_guard.get_path(), var_guard.name()))
                } else {
                    let mut var_guard = var.write();
                    var_guard.set_name("".to_string());
                    f.write_line(format_args!("[{}]", item_guard.get_path()))
                }
            }
            DataSource::Join { join_type, condition, left, right } => {
                left.fmt(f.clone(), context, aliases)?;
                {
                    let mut line = f.line()?;
                    line.write_str(match join_type {
                        JoinType::Cross => "CROSS JOIN",
                        JoinType::Left => "LEFT JOIN",
                        JoinType::Right => "RIGHT JOIN",
                    })?;
                    if let Some(condition) = condition {
                        line.write_str(" ON ")?;
                        condition.fmt(&mut line, context)?;
                    }
                }
                right.fmt(f, context, aliases)
            }
            DataSource::Selection { query, alias, var: _ } => {
                query.fmt(f.clone(), context)?;
                if aliases {
                    f.sub_block().write_line(format_args!("AS {}", alias))?;
                }
                Ok(())
            }
        }
    }
}
