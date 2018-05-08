use helpers::{
    ID,
    IDPull,
};
use helpers::SyncRef;
use lexeme_scanner::ItemPosition;
use language::{
    ItemPath,
    DataType,
    StatementResultType,
};
use project_analysis::{
    Module,
    SemanticError,
    SemanticItemType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionVariable {
    name: String,
    pos: ItemPosition,
    data_type: Option<StatementResultType>,
}

impl FunctionVariable {
    #[inline]
    fn new(pos: ItemPosition, name: String, data_type: Option<StatementResultType>) -> SyncRef<Self> {
        SyncRef::new(FunctionVariable {
            name,
            pos,
            data_type,
        })
    }
}

impl SyncRef<FunctionVariable> {
    pub fn property_type(&self, property_path: &ItemPath) -> Result<DataType, SemanticError> {
        let var = self.read();
        let stmt_type = match &var.data_type {
            &Some(ref var_type) => var_type,
            &None => return Err(SemanticError::variable_type_is_unknown(property_path.pos, var.name.clone())),
        };
        let var_type = match stmt_type {
            &StatementResultType::Data(ref data_type) => data_type,
            &StatementResultType::Table(_) => return Err(SemanticError::expected_item_of_another_type(
                property_path.pos,
                SemanticItemType::DataType,
                SemanticItemType::Table,
            )),
        };
        let result = var_type.property_type(property_path.pos, property_path.path.as_path())?;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionVariableScope {
    id: ID,
    parent: Option<ID>,
    context: SyncRef<FunctionContext>,
    variables: Vec<SyncRef<FunctionVariable>>,
}

impl FunctionVariableScope {
    #[inline]
    fn new(id: ID, parent: Option<ID>, context: SyncRef<FunctionContext>) -> SyncRef<Self> {
        SyncRef::new(FunctionVariableScope {
            id,
            parent,
            context,
            variables: Vec::new(),
        })
    }
}

impl SyncRef<FunctionVariableScope> {
    pub fn child(&self) -> Self {
        let scope = self.read();
        scope.context.new_scope(Some(scope.id))
    }
    pub fn get_variable(&self, name: &str) -> Option<SyncRef<FunctionVariable>> {
        let scope = self.read();
        {
            let found_in_self = scope.variables.iter()
                .find(|v| {
                    let var = v.read();
                    var.name == name
                });
            if let Some(var) = found_in_self {
                return Some(var.clone());
            }
        }
        scope.context
            .get_scope(scope.parent?)?
            .get_variable(name)
    }
    pub fn access_to_variable(&self, pos: ItemPosition, name: &str) -> Result<SyncRef<FunctionVariable>, SemanticError> {
        match self.get_variable(name) {
            Some(var) => Ok(var),
            None => Err(SemanticError::not_in_scope(pos, name.to_string())),
        }
    }
    pub fn new_variable(&self, pos: ItemPosition, name: String, data_type: Option<StatementResultType>) -> Result<SyncRef<FunctionVariable>, SemanticError> {
        if self.get_variable(name.as_str()).is_some() {
            return Err(SemanticError::duplicate_definition(pos, name, SemanticItemType::Variable));
        }
        let var = FunctionVariable::new(pos, name, data_type);
        self.write().variables.push(var.clone());
        Ok(var)
    }
    #[inline]
    pub fn context(&self) -> SyncRef<FunctionContext> {
        self.read().context.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionContext {
    module: SyncRef<Module>,
    scope_id_pull: IDPull,
    scopes: Vec<SyncRef<FunctionVariableScope>>,
    root: Option<ID>,
}

impl FunctionContext {
    #[inline]
    pub fn new(module: SyncRef<Module>) -> SyncRef<Self> {
        SyncRef::new(FunctionContext {
            module,
            scope_id_pull: IDPull::new(),
            scopes: Vec::new(),
            root: None,
        })
    }
}

impl SyncRef<FunctionContext> {
    fn new_scope(&self, parent_scope: Option<ID>) -> SyncRef<FunctionVariableScope> {
        let mut ctx = self.write();
        let scope = FunctionVariableScope::new(
            ctx.scope_id_pull.generate(),
            parent_scope,
            self.clone(),
        );
        ctx.scopes.push(scope.clone());
        scope
    }
    fn get_scope(&self, id: ID) -> Option<SyncRef<FunctionVariableScope>> {
        let ctx = self.read();
        let scope = ctx.scopes.iter()
            .find(|s| {
                let scope = s.read();
                scope.id == id
            })?
            .clone();
        Some(scope)
    }
    pub fn root(&self) -> SyncRef<FunctionVariableScope> {
        match self.read().root {
            Some(id) => self.get_scope(id)
                .expect("Root scope is not exists while its id was written"),
            None => {
                let result = self.new_scope(None);
                {
                    let mut ctx = self.write();
                    ctx.root = Some(result.read().id);
                }
                result
            }
        }
    }
    #[inline]
    pub fn module(&self) -> SyncRef<Module> {
        self.read().module.clone()
    }
}
