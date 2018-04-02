use indexmap::IndexMap;
use parser_basics::StaticIdentifier;
use super::module::Module;

#[derive(Debug)]
pub struct Project {
    modules: IndexMap<ModulePath, Module>,
}

pub type ModulePath = Vec<StaticIdentifier>;
pub type ModulePathSlice = [StaticIdentifier];

impl Project {
//    pub fn insert_module(&mut self, path: ModulePath, module: Module) -> ModuleId {
//        let module_id = self.modules.len();
//        self.modules.insert(path, module);
//        ModuleId { module_id }
//    }
//    pub fn find_module(&self, path: &ModulePathSlice) -> Option<ModuleId> {
//        self.modules.get_full(path)
//            .map(|(module_id, _, _)| ModuleId { module_id })
//    }
}
