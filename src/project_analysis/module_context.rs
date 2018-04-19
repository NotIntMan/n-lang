use std::collections::HashMap;
use helpers::sync_ref::SyncRef;
use helpers::path::{
    Path,
    PathBuf,
};
use project_analysis::Item;

#[derive(Debug)]
pub struct ModuleContext {
    items: HashMap<PathBuf, SyncRef<Item>>,
}

pub struct ErrorTypeNotFound;

impl ModuleContext {
    pub fn put_item(&mut self, name: Path, value: Item) -> SyncRef<Item> {
        let refer = SyncRef::new(value);
        self.items.insert(name.into(), refer.clone());
        refer
    }
    pub fn get_item(&mut self, name: &str) -> Option<SyncRef<Item>> {
        Some(
            self.items.get(name)?
                .clone()
        )
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
