use std::mem::replace;
use indexmap::IndexMap;
use helpers::path::{
    Path,
    PathBuf,
};
use helpers::sync_ref::SyncRef;
use helpers::resolve::Resolve;
use project_analysis::{
    Module,
    TextSource,
    SemanticError,
    UnresolvedModule,
};

#[derive(Debug)]
pub struct ProjectContext {
    modules: IndexMap<SyncRef<PathBuf>, ResolutionModuleState>,
    new_module_requested: bool,
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
    pub fn new() -> SyncRef<Self> {
        SyncRef::new(ProjectContext {
            modules: IndexMap::new(),
            new_module_requested: false,
        })
    }
    pub fn get_module(&self, path: Path) -> Option<&ResolutionModuleState> {
        for (item_path, item) in self.modules.iter() {
            let item_path = item_path.read();
            if item_path.as_path() == path {
                return Some(item);
            }
        }
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
        let mut result = Vec::new();
        for (module_path, module) in project.modules.iter_mut() {
            let new_state = match module {
                &mut ResolutionModuleState::Unresolved(ref module) => {
                    let mut project_context = (module_path.clone(), self.clone());
                    match module.resolve(&mut project_context) {
                        Ok(module) => ResolutionModuleState::Resolved(SyncRef::new(module)),
                        Err(mut errors) => {
//                            println!("Errors while resolving module {:#?}", errors);
                            result.append(&mut errors);
                            continue;
                        }
                    }
                }
                _ => continue,
            };
            replace(module, new_state);
        }
        result
    }
}

impl<S: TextSource> Resolve<S> for SyncRef<ProjectContext> {
    type Result = IndexMap<SyncRef<PathBuf>, SyncRef<Module>>;
    type Error = SemanticError;
    fn resolve(&self, ctx: &mut S) -> Result<Self::Result, Vec<Self::Error>> {
        let mut errors = Vec::new();
        loop {
            if !self.load_requested_modules(ctx) {
                break;
            }
            errors = self.resolution_step();
            if !self.read().new_module_requested {
                break;
            }
        }
        {
            let mut project = self.write();
            for (_, module) in project.modules.iter_mut() {
                match module {
                    &mut ResolutionModuleState::ParseFailed(ref mut parse_errors) => {
                        errors.append(parse_errors);
                    }
                    _ => continue,
                }
            }
        }
        let project = self.read();
        if errors.is_empty() {
            let mut result = IndexMap::new();
//            println!("Preparing Result::Ok of {:#?}", project.modules);
            for (path, module) in project.modules.iter() {
                match module {
                    &ResolutionModuleState::Resolved(ref module) => {
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
