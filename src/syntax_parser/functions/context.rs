use helpers::id_pull::{
    ID,
    IDPull,
};
use helpers::sync_ref::SyncRef;
use lexeme_scanner::ItemPosition;
use parser_basics::StaticIdentifier;
use syntax_parser::compound_types::DataType;
use syntax_parser::others::StaticPath;
use project_analysis::error::SemanticError;
use project_analysis::item::SemanticItemType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionVariable {
    name: StaticIdentifier,
    data_type: Option<DataType<'static>>,
}

impl FunctionVariable {
    #[inline]
    fn new(name: StaticIdentifier, data_type: Option<DataType<'static>>) -> SyncRef<Self> {
        SyncRef::new(FunctionVariable {
            name,
            data_type,
        })
    }
}

impl SyncRef<FunctionVariable> {
    pub fn type_of(&self, property_path: &StaticPath) -> Option<Result<DataType<'static>, SemanticError>> {
        let var = self.read();
        let mut var_type = match &var.data_type {
            &Some(ref var_type) => var_type,
            &None => return None,
        };
        let mut property_path_slice = property_path.path.as_slice();

        while let Some(name) = property_path_slice.first() {
            property_path_slice = &property_path_slice[1..];
            var_type = match var_type.prop(property_path.pos, name) {
                Ok(var_type) => var_type,
                Err(e) => return Some(Err(e)),
            };
        }

        Some(Ok(var_type.clone()))
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
    pub fn get_variable(&self, name: &StaticIdentifier) -> Option<SyncRef<FunctionVariable>> {
        let scope = self.read();
        {
            let found_in_self = scope.variables.iter()
                .find(|v| {
                    let var = v.read();
                    var.name == *name
                });
            if let Some(var) = found_in_self {
                return Some(var.clone());
            }
        }
        scope.context
            .get_scope(scope.parent?)?
            .get_variable(name)
    }
    pub fn access_to_variable(&self, pos: ItemPosition, name: &StaticIdentifier) -> Result<SyncRef<FunctionVariable>, SemanticError> {
        match self.get_variable(name) {
            Some(var) => Ok(var),
            None => Err(SemanticError::not_in_scope(pos, name.clone())),
        }
    }
    pub fn new_variable(&self, pos: ItemPosition, name: StaticIdentifier, data_type: Option<DataType<'static>>) -> Result<SyncRef<FunctionVariable>, SemanticError> {
        if self.get_variable(&name).is_some() {
            return Err(SemanticError::duplicate_definition(pos, name, SemanticItemType::Variable));
        }
        let var = FunctionVariable::new(name, data_type);
        self.write().variables.push(var.clone());
        Ok(var)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionContext {
    scope_id_pull: IDPull,
    scopes: Vec<SyncRef<FunctionVariableScope>>,
    root: Option<ID>,
}

impl FunctionContext {
    #[inline]
    pub fn new() -> SyncRef<Self> {
        SyncRef::new(FunctionContext {
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
            Some(id) => match self.get_scope(id) {
                Some(scope) => scope,
                None => panic!("Root scope is not exists while its id was written"),
            },
            None => {
                let result = self.new_scope(None);
                {
                    let mut ctx = self.write();
                    let root = result.read();
                    ctx.root = Some(root.id);
                }
                result
            }
        }
    }
}
