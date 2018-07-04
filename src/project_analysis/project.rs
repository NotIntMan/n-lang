use std::mem::replace;
use std::sync::Arc;
use indexmap::IndexMap;
use helpers::{
    Path,
    PathBuf,
};
use helpers::{
    Resolve,
    SyncRef,
};
use lexeme_scanner::ItemPosition;
use language::{
    BinaryOperator,
    DataType,
    PostfixUnaryOperator,
    PrefixUnaryOperator,
};
use project_analysis::{
    Item,
    Module,
    TextSource,
    SemanticError,
    StdLib,
    StdLibBinaryOperation,
    StdLibFunction,
    StdLibPostfixUnaryOperation,
    StdLibPrefixUnaryOperation,
    UnresolvedModule,
};

#[derive(Debug)]
pub struct ProjectContext {
    modules: IndexMap<SyncRef<PathBuf>, ResolutionModuleState>,
    new_module_requested: bool,
    new_module_resolved: bool,
    stdlib: SyncRef<StdLib>,
}

#[derive(Debug, Clone)]
pub enum ResolutionModuleState {
    Requested,
    LoadFailed,
    ParseFailed(Vec<SemanticError>),
    Unresolved(UnresolvedModule),
    Resolved(SyncRef<Module>),
}

impl ProjectContext {
    #[inline]
    pub fn new(stdlib: SyncRef<StdLib>) -> SyncRef<Self> {
        SyncRef::new(ProjectContext {
            modules: IndexMap::new(),
            new_module_requested: false,
            new_module_resolved: false,
            stdlib,
        })
    }
    pub fn get_module(&self, path: Path) -> Option<&ResolutionModuleState> {
        println!("Getting module {:?} from project's context required", path);
        for (item_path, item) in self.modules.iter() {
            let item_path = item_path.read();
            if item_path.as_path() == path {
                println!("Found {:?}", item);
                return Some(item);
            }
        }
        println!("Nothing was found");
        None
    }
}

impl SyncRef<ProjectContext> {
    pub fn request_resolving_module(&self, path: Path) {
        if self.read().get_module(path).is_none() {
            let mut project = self.write();
            project.modules.insert(SyncRef::new(path.into()), ResolutionModuleState::Requested);
            project.new_module_requested = true;
        }
    }
    fn load_requested_modules<S: TextSource>(&self, source: &S) -> bool {
        let mut new_modules_loaded = false;
        let mut project = self.write();
        for (module_path, module) in project.modules.iter_mut() {
            let new_state = match module {
                &mut ResolutionModuleState::Requested => {
                    let module_path = module_path.read();
                    match source.get_text(module_path.as_path()) {
                        Some(text) => match UnresolvedModule::new(text) {
                            Ok(module) => {
                                new_modules_loaded = true;
                                ResolutionModuleState::Unresolved(module)
                            }
                            Err(errors) => ResolutionModuleState::ParseFailed(errors),
                        },
                        None => ResolutionModuleState::LoadFailed,
                    }
                }
                _ => continue,
            };
            replace(module, new_state);
        }
        new_modules_loaded
    }
    fn resolution_step(&self) -> Vec<SemanticError> {
        let mut project = self.write();
        project.new_module_requested = false;
        let mut new_module_resolved = false;
        let mut result = Vec::new();
        for (module_path, module) in project.modules.iter_mut() {
            let new_state = match module {
                ResolutionModuleState::Unresolved(module) => {
                    let mut project_context = (module_path.clone(), self.clone());
                    println!("Resolving module {:?}", *module_path.read());
                    match module.resolve(&mut project_context) {
                        Ok(module) => {
                            new_module_resolved = true;
                            println!("Resolved");
                            ResolutionModuleState::Resolved(module)
                        }
                        Err(mut errors) => {
                            println!("Failed with: {:#?}", errors);
                            result.append(&mut errors);
                            continue;
                        }
                    }
                }
                _ => continue,
            };
            replace(module, new_state);
        }
        project.new_module_resolved = new_module_resolved;
        result
    }
    #[inline]
    fn need_more_resolution_steps(&self) -> bool {
        let project = self.read();
        project.new_module_resolved || project.new_module_requested
    }
    pub fn get_module(&self, path: Path) -> Option<SyncRef<Module>> {
        {
            let project = self.read();
            match project.get_module(path) {
                Some(module_state) => match module_state {
                    ResolutionModuleState::Resolved(module) => return Some(module.clone()),
                    _ => return None,
                }
                _ => {}
            }
        }
        let mut project = self.write();
        project.modules.insert(SyncRef::new(path.into()), ResolutionModuleState::Requested);
        project.new_module_requested = true;
        None
    }
    pub fn resolve_item(&self, mut path: Path) -> Option<SyncRef<Item>> {
        let mut module_path = path;
        let module = loop {
            if let Some(module) = self.get_module(module_path) {
                break module
            }
            if module_path.is_empty() {
                return None;
            }
            module_path.pop_right();
        };
        for _ in module_path {
            path.pop_left();
        }
        module.get_item(path, &mut Vec::new())
    }
    pub fn resolve_binary_operation(&self, pos: ItemPosition, operator: BinaryOperator, left: &DataType, right: &DataType) -> Result<Arc<StdLibBinaryOperation>, SemanticError> {
        match self.read().stdlib.resolve_binary_operation(operator, left, right) {
            Some(op) => Ok(op),
            None => Err(SemanticError::binary_operation_cannot_be_performed(pos, operator, left.clone(), right.clone())),
        }
    }
    pub fn resolve_postfix_unary_operation(&self, pos: ItemPosition, operator: PostfixUnaryOperator, input: &DataType) -> Result<Arc<StdLibPostfixUnaryOperation>, SemanticError> {
        match self.read().stdlib.resolve_postfix_unary_operation(operator, input) {
            Some(op) => Ok(op),
            None => Err(SemanticError::postfix_unary_operation_cannot_be_performed(pos, operator, input.clone())),
        }
    }
    pub fn resolve_prefix_unary_operation(&self, pos: ItemPosition, operator: PrefixUnaryOperator, input: &DataType) -> Result<Arc<StdLibPrefixUnaryOperation>, SemanticError> {
        match self.read().stdlib.resolve_prefix_unary_operation(operator, input) {
            Some(op) => Ok(op),
            None => Err(SemanticError::prefix_unary_operation_cannot_be_performed(pos, operator, input.clone())),
        }
    }
    #[inline]
    pub fn resolve_stdlib_function(&self, name: &str) -> Option<Arc<StdLibFunction>> {
        self.read().stdlib.resolve_function(name)
    }
}

impl<S: TextSource> Resolve<S> for SyncRef<ProjectContext> {
    type Result = IndexMap<SyncRef<PathBuf>, SyncRef<Module>>;
    type Error = SemanticError;
    fn resolve(&self, ctx: &S) -> Result<Self::Result, Vec<Self::Error>> {
        let mut errors = Vec::new();
        loop {
            if !(
                self.load_requested_modules(ctx)
                    ||
                    self.need_more_resolution_steps()
            ) {
                println!("Breaking resolution cycle");
                break;
            }
            errors = self.resolution_step();
            println!("Continuing resolution cycle");
        }
        {
            let mut project = self.write();
            for (_, module) in project.modules.iter_mut() {
                match module {
                    ResolutionModuleState::ParseFailed(parse_errors) => {
                        errors.append(parse_errors);
                    }
                    _ => continue,
                }
            }
        }
        let project = self.read();
        if errors.is_empty() {
            let mut result = IndexMap::new();
            for (path, module) in project.modules.iter() {
                match module {
                    ResolutionModuleState::Resolved(module) => {
                        result.insert(path.clone(), module.clone());
                    }
                    _ => {}
                }
            }
            Ok(result)
        } else {
            Err(errors)
        }
    }
}
