use std::collections::HashMap;
use std::sync::Arc;
use helpers::group::Group;
use helpers::loud_rw_lock::LoudRwLock;
use parser_basics::StaticIdentifier;
use syntax_parser::others::Path;
use super::module::{
    Module,
    ModuleRef,
};
use super::source::TextSource;
use super::error::SemanticError;

#[derive(Debug)]
pub struct Project {
    modules: HashMap<ModulePath, ModuleRef>,
}

pub type ProjectRef = Arc<LoudRwLock<Project>>;

pub type ModulePath = Vec<StaticIdentifier>;
pub type ModulePathSlice = [StaticIdentifier];

impl Project {
    pub fn new(root: ModuleRef) -> Self {
        let mut modules = HashMap::with_capacity(1);
        modules.insert(vec![], root);
        Project {
            modules,
        }
    }
    pub fn try_init<S: TextSource>(source: &S) -> Result<ProjectRef, Group<SemanticError>> {
        let (module, _) = Module::try_load(source, Path::for_root())?;
        let project = Project::new(module);
        Ok(Arc::new(LoudRwLock::new(project, "Project was poisoned!")))
    }
    pub fn insert_module(&mut self, path: ModulePath, module: ModuleRef) {
        self.modules.insert(path, module);
    }
    pub fn get_module(&self, path: &ModulePathSlice) -> Option<ModuleRef> {
        self.modules.get(path)
            .map(Clone::clone)
    }
    #[inline]
    pub fn get_root(&self) -> ModuleRef {
        self.get_module(&[][..])
            .expect("Project do not contain root module")
    }
}
