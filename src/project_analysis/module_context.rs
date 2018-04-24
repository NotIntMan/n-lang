use std::collections::HashMap;
use std::fmt;
use helpers::sync_ref::SyncRef;
use helpers::path::PathBuf;
use syntax_parser::others::ItemPath;
use project_analysis::{
    Item,
    SemanticError,
    ProjectContext,
};

pub struct ModuleContext {
    items: HashMap<String, SyncRef<Item>>,
    module_path: SyncRef<PathBuf>,
    project: SyncRef<ProjectContext>,
}

pub struct ErrorTypeNotFound;

impl ModuleContext {
    #[inline]
    pub fn new(module_path: SyncRef<PathBuf>, project: SyncRef<ProjectContext>) -> Self {
        ModuleContext {
            items: HashMap::new(),
            module_path,
            project,
        }
    }
    pub fn put_item(&mut self, name: &str, value: Item) -> SyncRef<Item> {
        let refer = SyncRef::new(value);
        self.items.insert(name.into(), refer.clone());
        refer
    }
    pub fn get_item(&mut self, path: &ItemPath) -> Result<SyncRef<Item>, Vec<SemanticError>> {
        let (search_name, rest_path) = path.path.as_path().pop_left();
        let search_name = match search_name {
            Some(search_name) => search_name,
            None => unimplemented!(),
        };
        for (item_name, item) in self.items.iter() {
            if item_name == search_name {
                match item.get_item(rest_path) {
                    Some(item) => return Ok(item),
                    None => break,
                }
            }
        }
        Err(vec![
            SemanticError::unresolved_item(path.pos, path.path.clone()),
        ])
    }
//    pub fn put_type(&mut self, name: &str, data_type: DataType<'static>) {
//        self.types.insert(name.to_string(), data_type);
//    }
//    pub fn get_type(&mut self, name: &str) -> Result<DataType<'static>, ErrorTypeNotFound> {
//        match self.types.get(name) {
//            Some(res) => Ok(res.clone()),
//            None => Err(ErrorTypeNotFound),
//        }
//    }
}

impl fmt::Debug for ModuleContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ModuleContext")
            .field("items", &self.items)
            .field("module_path", &self.module_path)
            .finish()
    }
}
