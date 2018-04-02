use std::sync::Arc;
use syntax_parser::modules::ModuleDefinitionItem;
use super::project::Project;
use super::source::Text;
use super::item::Item;

#[derive(Debug)]
pub struct Module {
    text: Arc<Text>,
    items: Vec<Item>,
}

impl Module {
    pub fn from_def(text: Arc<Text>, items: Vec<ModuleDefinitionItem>) -> Self {
        Module {
            text,
            items: items.into_iter()
                .map(|def| Item::from_def(def))
                .collect(),
        }
    }
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
