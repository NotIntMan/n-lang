use helpers::{
    PathBuf,
    Resolve,
    SyncRef,
};
use parser_basics::Identifier;
use language::{
    DataType,
    Expression,
    ExpressionAST,
    ItemPath,
    Selection,
    SelectionAST,
};
use project_analysis::{
    FunctionVariable,
    FunctionVariableScope,
    Item,
    SemanticError,
    SemanticItemType,
};

#[derive(Debug, Clone, PartialEq)]
pub enum JoinConditionAST<'source> {
    Expression(ExpressionAST<'source>),
    Using(Vec<ItemPath>),
    Natural,
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for JoinConditionAST<'source> {
    type Result = JoinCondition;
    type Error = SemanticError;
    fn resolve(&self, scope: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let result = match self {
            JoinConditionAST::Expression(expr) => JoinCondition::Expression(expr.resolve(scope)?),
            JoinConditionAST::Using(paths) => JoinCondition::Using(paths.clone()),
            JoinConditionAST::Natural => JoinCondition::Natural,
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinCondition {
    Expression(Expression),
    // TODO Специальная проверка синтаксиса JOIN ... USING (...)
    Using(Vec<ItemPath>),
    // TODO Специальная проверка синтаксиса JOIN ... NATURAL
    Natural,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Cross,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataSourceAST<'source> {
    Table {
        name: ItemPath,
        alias: Option<Identifier<'source>>,
    },
    Join {
        join_type: JoinType,
        condition: Option<JoinConditionAST<'source>>,
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
                        let var_data_type = var.property_type(&ItemPath {
                            pos,
                            path: PathBuf::empty(),
                        })?;
                        let entity_type = match var_data_type {
                            DataType::Array(entity_type) => (*entity_type).clone(),
                            _ => return SemanticError::not_allowed_here(pos, "not array variable").into_err_vec(),
                        };
                        let new_var_name = match alias {
                            Some(alias) => alias.text(),
                            None => name,
                        };
                        scope.new_variable(pos, new_var_name.to_string(), Some(entity_type))?;
                        return Ok(DataSource::Variable { var });
                    }
                }
                match scope.module().get_item(name.path.as_path(), &mut Vec::new()) {
                    Some(item) => {
                        {
                            let mut item = item.write();
                            let item_type = item.get_type();
                            match item.get_table_mut() {
                                Some(table) => {
                                    let new_var_name = match alias {
                                        Some(alias) => alias.text(),
                                        None => name.path.as_path()
                                            .pop_right()
                                            .expect("Item's path should not be null"),
                                    };
                                    scope.new_variable(
                                        pos,
                                        new_var_name.to_string(),
                                        Some(table.make_entity_type()),
                                    )?;
                                }
                                None => return SemanticError::expected_item_of_another_type(
                                    name.pos,
                                    SemanticItemType::Table,
                                    item_type,
                                )
                                    .into_err_vec(),
                            }
                        }
                        Ok(DataSource::Table { item })
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
                scope.new_variable(
                    alias.item_pos(),
                    alias.to_string(),
                    Some(query.result_data_type.clone()),
                )?;
                Ok(DataSource::Selection { query, alias: alias.to_string() })
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
    },
    Join {
        join_type: JoinType,
        condition: Option<JoinCondition>,
        left: Box<DataSource>,
        right: Box<DataSource>,
    },
    Selection {
        query: Box<Selection>,
        alias: String,
    },
}
