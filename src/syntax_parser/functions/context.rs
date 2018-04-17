use helpers::id_pull::{
    ID,
    IDPull,
};
use helpers::sync_ref::SyncRef;
use parser_basics::StaticIdentifier;
use syntax_parser::compound_types::DataType;

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
    pub fn new_variable(&self, name: StaticIdentifier, data_type: Option<DataType<'static>>) -> SyncRef<FunctionVariable> {
        let mut scope = self.write();
        unimplemented!()
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
            None,
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
