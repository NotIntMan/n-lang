use helpers::{
    ID,
    IDPull,
    Path,
};
use helpers::SyncRef;
use language::DataType;
use lexeme_scanner::ItemPosition;
use project_analysis::{
    Module,
    ProjectContext,
    SemanticError,
    SemanticItemType,
};
use std::{
    fmt,
    mem::replace,
};

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionVariable {
    name: String,
    pos: ItemPosition,
    data_type: Option<DataType>,
    is_read_only: bool,
    is_argument: bool,
}

impl FunctionVariable {
    #[inline]
    fn new(pos: ItemPosition, name: String, data_type: Option<DataType>) -> SyncRef<Self> {
        SyncRef::new(FunctionVariable {
            name,
            pos,
            data_type,
            is_read_only: false,
            is_argument: false,
        })
    }
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    #[inline]
    pub fn set_name(&mut self, new_name: String) -> String {
        replace(&mut self.name, new_name)
    }
    #[inline]
    pub fn data_type(&self) -> Option<&DataType> {
        match &self.data_type {
            Some(data_type) => Some(data_type),
            None => None,
        }
    }
    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.is_read_only
    }
    #[inline]
    pub fn make_read_only(&mut self) {
        self.is_read_only = true
    }
    #[inline]
    pub fn is_argument(&self) -> bool {
        self.is_argument
    }
    #[inline]
    pub fn mark_as_argument(&mut self) {
        self.is_argument = true
    }
}

impl SyncRef<FunctionVariable> {
    pub fn data_type(&self, pos: ItemPosition) -> Result<DataType, SemanticError> {
        match self.read().data_type() {
            Some(var_type) => Ok(var_type.clone()),
            None => Err(SemanticError::variable_type_is_unknown(pos, self.read().name.clone())),
        }
    }
    pub fn property_type(&self, pos: ItemPosition, property_path: Path) -> Result<DataType, SemanticError> {
        Ok(
            self.data_type(pos)?
                .property_type(pos, property_path)?
        )
    }
    #[inline]
    pub fn replace_data_type(&self, data_type: DataType) {
        self.write().data_type = Some(data_type);
    }
    #[inline]
    pub fn is_read_only(&self) -> bool {
        self.read().is_read_only()
    }
    #[inline]
    pub fn make_read_only(&self) {
        self.write().make_read_only()
    }
    #[inline]
    pub fn is_argument(&self) -> bool {
        self.read().is_argument()
    }
    #[inline]
    pub fn mark_as_argument(&self) {
        self.write().mark_as_argument()
    }
}

#[derive(Clone, PartialEq)]
pub struct FunctionVariableScope {
    id: ID,
    parent: Option<ID>,
    context: SyncRef<FunctionContext>,
    variables: Vec<SyncRef<FunctionVariable>>,
    is_aggregate: bool,
    is_lite_weight: bool,
}

impl FunctionVariableScope {
    #[inline]
    fn new(id: ID, parent: Option<ID>, context: SyncRef<FunctionContext>) -> SyncRef<Self> {
        SyncRef::new(FunctionVariableScope {
            id,
            parent,
            context,
            variables: Vec::new(),
            is_aggregate: false,
            is_lite_weight: false,
        })
    }
    pub fn variables(&self) -> &[SyncRef<FunctionVariable>] {
        self.variables.as_slice()
    }
    pub fn find_variable(&self, name: &str) -> Option<&SyncRef<FunctionVariable>> {
        self.variables.iter()
            .find(|v| v.read().name() == name)
    }
}

impl SyncRef<FunctionVariableScope> {
    fn _child(&self, is_aggregate_overload: bool, is_lite_weight_overload: bool) -> Self {
        let scope = self.read();
        let result = scope.context.new_scope(Some(scope.id));
        {
            let mut child = result.write();
            child.is_aggregate = is_aggregate_overload || scope.is_aggregate;
            child.is_lite_weight = is_lite_weight_overload || scope.is_lite_weight;
        }
        result
    }
    #[inline]
    pub fn child(&self) -> Self {
        self._child(false, false)
    }
    #[inline]
    pub fn aggregate_child(&self) -> Self {
        self._child(true, false)
    }
    #[inline]
    pub fn lite_weight_child(&self) -> Self {
        self._child(false, true)
    }
    #[inline]
    pub fn parent(&self) -> Option<SyncRef<FunctionVariableScope>> {
        let scope = self.read();
        scope.context.get_scope(scope.parent?)
    }
    pub fn get_variable(&self, name: &str) -> Option<SyncRef<FunctionVariable>> {
        if let Some(var) = self.read().find_variable(name) {
            return Some(var.clone());
        }
        self.parent()?
            .get_variable(name)
    }
    pub fn access_to_variable(&self, pos: ItemPosition, name: &str) -> Result<SyncRef<FunctionVariable>, SemanticError> {
        match self.get_variable(name) {
            Some(var) => Ok(var),
            None => Err(SemanticError::not_in_scope(pos, name.to_string())),
        }
    }
    pub fn new_variable(&self, pos: ItemPosition, name: String, data_type: Option<DataType>) -> Result<SyncRef<FunctionVariable>, SemanticError> {
        if self.read().find_variable(name.as_str()).is_some() {
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
    #[inline]
    pub fn module(&self) -> SyncRef<Module> { self.context().module() }
    #[inline]
    pub fn project(&self) -> SyncRef<ProjectContext> { self.module().project() }
    #[inline]
    pub fn is_aggregate(&self) -> bool {
        self.read().is_aggregate
    }
    #[inline]
    pub fn is_lite_weight(&self) -> bool {
        self.read().is_lite_weight
    }
}

impl fmt::Debug for FunctionVariableScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FunctionVariableScope")
            .field("id", &self.id)
            .field("parent", &self.parent)
            .field("variables", &self.variables)
            .field("is_aggregate", &self.is_aggregate)
            .field("is_lite_weight", &self.is_lite_weight)
            .finish()
    }
}

#[derive(Clone, PartialEq)]
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
    #[inline]
    pub fn project(&self) -> SyncRef<ProjectContext> { self.module().project() }
}

impl fmt::Debug for FunctionContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FunctionContext")
            .field("scopes", &self.scopes)
            .field("root", &self.root)
            .finish()
    }
}
