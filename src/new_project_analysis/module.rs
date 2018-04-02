use std::sync::Arc;
use indexmap::IndexMap;
use super::project::Project;
use super::source::Text;
use super::item::Item;

#[derive(Debug)]
pub struct Module {
    text: Arc<Text>,
    items: IndexMap<String, Item>,
    aliases: Vec<ModuleId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ModuleId {
    pub module_id: usize,
}

#[derive(Debug)]
pub struct ModuleContext<'a> {
    project: &'a Project,
    module_id: ModuleId,
}
