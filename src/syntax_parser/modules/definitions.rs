use helpers::into_static::IntoStatic;
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use syntax_parser::compound_types::{
    Attribute,
    DataType,
    Field,
};
use syntax_parser::functions::FunctionDefinition;
use syntax_parser::others::{
    Path,
    StaticPath,
};
use project_analysis::resolve::ResolveContext;
use project_analysis::item::ItemBody;
use project_analysis::error::SemanticError;
use project_analysis::module::ModuleRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTypeDefinition<'source> {
    pub name: Identifier<'source>,
    pub body: DataType<'source>,
}

impl<'source> IntoStatic for DataTypeDefinition<'source> {
    type Result = DataTypeDefinition<'static>;
    fn into_static(self) -> Self::Result {
        let DataTypeDefinition { name, body } = self;
        DataTypeDefinition {
            name: name.into_static(),
            body: body.into_static(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDefinition<'source> {
    pub name: Identifier<'source>,
    pub pos: ItemPosition,
    pub body: Vec<(Identifier<'source>, Field<'source>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalItemTail<'source> {
    None,
    Asterisk,
    Alias(Identifier<'source>),
}

impl<'source> IntoStatic for ExternalItemTail<'source> {
    type Result = ExternalItemTail<'static>;
    fn into_static(self) -> Self::Result {
        match self {
            ExternalItemTail::None => ExternalItemTail::None,
            ExternalItemTail::Asterisk => ExternalItemTail::Asterisk,
            ExternalItemTail::Alias(identifier) => ExternalItemTail::Alias(identifier.into_static()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalItemImport<'source> {
    pub path: Path<'source>,
    pub tail: ExternalItemTail<'source>,
}

impl ExternalItemImport<'static> {
    pub fn try_semantic_resolve(&mut self, context: &mut ResolveContext) -> Option<ItemBody> {
        match context.resolve_item(&self.path) {
            Ok(item) => match &self.tail {
                &ExternalItemTail::None => Some(ItemBody::ImportItem(
                    self.path.path.last()
                        .expect("Path should not be empty")
                        .clone(),
                    item,
                )),
                &ExternalItemTail::Alias(ref alias) => Some(ItemBody::ImportItem(
                    alias.clone(),
                    item,
                )),
                &ExternalItemTail::Asterisk => match item.get_module(self.path.pos) {
                    Ok(module) => Some(ItemBody::ModuleReference(module)),
                    Err(err) => {
                        context.throw_error(err);
                        None
                    }
                },
            },
            Err(err) => {
                context.throw_error(err);
                context.request_dependency(self.path.clone());
                None
            }
        }
    }
    pub fn try_put_dependency(&self, dependency: &StaticPath, module: &ModuleRef) -> Result<Option<ItemBody>, SemanticError> {
        let dependency_len = dependency.path.len();
        println!("Comparing paths (begin of {:?} and {:?}", dependency.path, self.path.path);
        if (self.path.path.len() >= dependency_len)
            &&
            (dependency.path.as_slice() == &self.path.path[..dependency_len]) {
            println!("Begin of dependency's path is equal to import's path. Trying to find item in dependency.");
            let item_path = &self.path.path[dependency_len..];
            match module.find_item(item_path) {
                Some(item) => {
                    println!("Item found, putting {:?}", item);
                    match &self.tail {
                        &ExternalItemTail::None => {
                            let name = self.path.path.last()
                                .expect("Path should not be empty!")
                                .clone();
                            Ok(Some(ItemBody::ImportItem(name, item)))
                        }
                        &ExternalItemTail::Alias(ref alias) => {
                            let name = alias.clone();
                            Ok(Some(ItemBody::ImportItem(name, item)))
                        }
                        &ExternalItemTail::Asterisk => {
                            match item.get_module(self.path.pos) {
                                Ok(module) => Ok(Some(ItemBody::ModuleReference(module))),
                                Err(err) => Err(err),
                            }
                        }
                    }
                }
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}

impl<'source> IntoStatic for ExternalItemImport<'source> {
    type Result = ExternalItemImport<'static>;
    fn into_static(self) -> Self::Result {
        let ExternalItemImport { path, tail } = self;
        ExternalItemImport {
            path: path.into_static(),
            tail: tail.into_static(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleDefinitionValue<'source> {
    DataType(DataTypeDefinition<'source>),
    Table(TableDefinition<'source>),
    Function(FunctionDefinition<'source>),
    Module(ModuleDefinition<'source>),
    Import(ExternalItemImport<'source>),
}

impl<'source> IntoStatic for ModuleDefinitionValue<'source> {
    type Result = ModuleDefinitionValue<'static>;
    fn into_static(self) -> Self::Result {
        match self {
            ModuleDefinitionValue::DataType(value) => ModuleDefinitionValue::DataType(value.into_static()),
            ModuleDefinitionValue::Import(value) => ModuleDefinitionValue::Import(value.into_static()),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDefinitionItem<'source> {
    pub public: bool,
    pub attributes: Vec<Attribute<'source>>,
    pub value: ModuleDefinitionValue<'source>,
}

impl<'source> IntoStatic for ModuleDefinitionItem<'source> {
    type Result = ModuleDefinitionItem<'static>;
    fn into_static(self) -> Self::Result {
        let ModuleDefinitionItem {
            public, attributes, value
        } = self;
        ModuleDefinitionItem {
            public,
            attributes: attributes.into_static(),
            value: value.into_static(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDefinition<'source> {
    pub name: Identifier<'source>,
    pub items: Vec<ModuleDefinitionItem<'source>>,
}
