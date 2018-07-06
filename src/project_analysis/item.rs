use std::fmt;
use helpers::{
    Path,
    PathBuf,
    SyncRef,
};
use language::{
    DataTypeDefinition,
    FunctionDefinition,
    TableDefinition,
};
use project_analysis::Module;

#[derive(Clone)]
pub struct Item {
    parent: SyncRef<Module>,
    body: ItemBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemBody {
    DataType {
        def: DataTypeDefinition,
    },
    ModuleReference {
        module: SyncRef<Module>,
    },
    Table {
        def: TableDefinition,
        entity: SyncRef<Item>,
        primary_key: SyncRef<Item>,
    },
    Function {
        def: FunctionDefinition,
    },
}

impl Item {
    #[inline]
    pub fn data_type(parent: SyncRef<Module>, def: DataTypeDefinition) -> Self {
        Item {
            parent,
            body: ItemBody::DataType { def },
        }
    }
    #[inline]
    pub fn module_ref(module: SyncRef<Module>) -> Self {
        Item {
            parent: module.clone(),
            body: ItemBody::ModuleReference { module },
        }
    }
    #[inline]
    pub fn function(parent: SyncRef<Module>, def: FunctionDefinition) -> Self {
        Item {
            parent,
            body: ItemBody::Function { def },
        }
    }
    #[inline]
    pub fn table(parent: SyncRef<Module>, mut def: TableDefinition) -> Self {
        let entity = SyncRef::new(Item::data_type(parent.clone(), DataTypeDefinition {
            name: format!("{}::entity", def.name),
            body: def.make_entity_type(),
        }));
        let primary_key = SyncRef::new(Item::data_type(parent.clone(), DataTypeDefinition {
            name: format!("{}::primary_key", def.name),
            body: def.make_primary_key_type(),
        }));
        Item {
            parent,
            body: ItemBody::Table { def, entity, primary_key },
        }
    }
    #[inline]
    pub fn get_type(&self) -> SemanticItemType {
        match &self.body {
            ItemBody::DataType { def: _ } => SemanticItemType::DataType,
            ItemBody::ModuleReference { module: _ } => SemanticItemType::Module,
            ItemBody::Table { def: _, entity: _, primary_key: _ } => SemanticItemType::Table,
            ItemBody::Function { def: _ } => SemanticItemType::Function,
        }
    }
    #[inline]
    pub fn get_data_type(&self) -> Option<&DataTypeDefinition> {
        match &self.body {
            ItemBody::DataType { def } => Some(def),
            _ => None,
        }
    }
    #[inline]
    pub fn get_module_ref(&self) -> Option<&SyncRef<Module>> {
        match &self.body {
            ItemBody::ModuleReference { module } => Some(module),
            _ => None,
        }
    }
    #[inline]
    pub fn get_function(&self) -> Option<&FunctionDefinition> {
        match &self.body {
            ItemBody::Function { def } => Some(def),
            _ => None,
        }
    }
    #[inline]
    pub fn get_table(&self) -> Option<&TableDefinition> {
        match &self.body {
            ItemBody::Table { def, entity: _, primary_key: _ } => Some(def),
            _ => None,
        }
    }
    #[inline]
    pub fn get_table_mut(&mut self) -> Option<&mut TableDefinition> {
        match &mut self.body {
            ItemBody::Table { def, entity: _, primary_key: _ } => Some(def),
            _ => None,
        }
    }
    pub fn get_path(&self) -> PathBuf {
        let name = match &self.body {
            ItemBody::DataType { def } => def.name.as_str(),
            ItemBody::ModuleReference { module } => {
                let module = module.read();
                let path = module.path().read();
                return path.clone();
            }
            ItemBody::Table { def, .. } => def.name.as_str(),
            ItemBody::Function { def } => def.name.as_str(),
        };
        let parent = self.parent.read();
        let path = parent.path().read();
        let mut result = path.clone();
        result.push(name);
        result
    }
    #[inline]
    pub fn body(&self) -> &ItemBody {
        &self.body
    }
}

impl SyncRef<Item> {
    pub fn get_item(&self, path: Path, search_route: &mut Vec<SyncRef<Module>>) -> Option<Self> {
        if path.is_empty() {
            return Some(self.clone());
        }
        let item = self.read();
        match &item.body {
            ItemBody::DataType { def: _ } => {}
            ItemBody::ModuleReference { module } => {
                return module.get_item(path, search_route);
            }
            ItemBody::Function { def: _ } => {}
            ItemBody::Table { def: _, entity, primary_key } => if let Some(name) = path.the_only() {
                match name {
                    "entity" => return Some(entity.clone()),
                    "primary_key" => return Some(primary_key.clone()),
                    _ => {}
                }
            }
        }
        None
    }
    #[inline]
    pub fn get_type(&self) -> SemanticItemType {
        self.read().get_type()
    }
}

impl PartialEq for Item {
    fn eq(&self, rhs: &Self) -> bool {
        self.parent.is_same_ref(&rhs.parent)
            &&
            self.body == rhs.body
    }
}

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "Item::{:#?}", self.body)
        } else {
            write!(f, "Item::{:?}", self.body)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticItemType {
    Definition,
    Field,
    DataType,
    Module,
    Table,
    Variable,
    Function,
}

impl SemanticItemType {
    pub fn get_description(&self) -> &'static str {
        match self {
            &SemanticItemType::Definition => "definition",
            &SemanticItemType::Field => "field",
            &SemanticItemType::DataType => "data type",
            &SemanticItemType::Module => "module",
            &SemanticItemType::Table => "table",
            &SemanticItemType::Variable => "variable",
            &SemanticItemType::Function => "function",
        }
    }
}

impl fmt::Display for SemanticItemType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_description())
    }
}
