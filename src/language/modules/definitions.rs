use indexmap::IndexMap;
//use helpers::IntoStatic;
use helpers::{
    as_unique_identifier,
    Resolve,
    SyncRef,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use language::{
    Attribute,
    AttributeAST,
    CompoundDataType,
    DataTypeAST,
    DataType,
    FieldAST,
    Field,
//    find_attribute,
    FunctionDefinitionAST,
    ItemPath,
//    StaticPath,
};
//use project_analysis::resolve::{
//    ResolveContext,
////    SemanticResolve,
//};
use project_analysis::{
    Item,
    Module,
    SemanticItemType,
    SemanticError,
//    ItemBody,
//    ItemRef,
};
//use project_analysis::module::ModuleRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTypeDefinitionAST<'source> {
    pub name: Identifier<'source>,
    pub body: DataTypeAST<'source>,
}

impl<'source> Resolve<SyncRef<Module>> for DataTypeDefinitionAST<'source> {
    type Result = Item;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        let body = self.body.resolve(ctx)?;
        let def = DataTypeDefinition {
            name: self.name.to_string(),
            body,
        };
        Ok(Item::data_type(def))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataTypeDefinition {
    pub name: String,
    pub body: DataType,
}

//impl<'source> IntoStatic for DataTypeDefinition<'source> {
//    type Result = DataTypeDefinition<'static>;
//    fn into_static(self) -> Self::Result {
//        let DataTypeDefinition { name, body } = self;
//        DataTypeDefinition {
//            name: name.into_static(),
//            body: body.into_static(),
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDefinitionAST<'source> {
    pub name: Identifier<'source>,
    pub pos: ItemPosition,
    pub body: Vec<(Identifier<'source>, FieldAST<'source>)>,
}

impl<'source> Resolve<SyncRef<Module>> for TableDefinitionAST<'source> {
    type Result = TableDefinition;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        let body = match as_unique_identifier(self.body.clone()) {
            Ok(map) => map,
            Err(name) => return SemanticError::duplicate_definition(
                name.item_pos(),
                name.text().to_string(),
                SemanticItemType::Field,
            )
                .into_err_vec(),
        }
            .resolve(ctx)?;
        Ok(TableDefinition {
            name: self.name.to_string(),
            pos: self.pos,
            body,
        })
    }
}

impl<'source> TableDefinitionAST<'source> {
//    pub fn make_primary_key(&self) -> Result<ItemRef, SemanticError> {
//        let fields = {
//            let mut fields = Vec::new();
//            for body_item in self.body.iter() {
//                let (ref name, ref field) = *body_item;
//                if let Some(_) = find_attribute(&field.attributes, "primary_key") {
//                    // TODO Сделать подсказку о том, что аргументы для primary_key не нужны
//                    fields.push((name.clone(), field.clone()));
//                }
//            }
//            fields
//        };
//        if fields.is_empty() {
//            return Err(SemanticError::empty_primary_key(self.pos));
//        }
//        let name = {
//            let mut name = self.name.clone();
//            {
//                let string = name.get_mut_text();
//                string.push_str("::primary_key");
//            }
//            name
//        };
//        let data_type = DataTypeDefinition {
//            name,
//            body: DataType::Compound(CompoundDataType::Structure(fields)),
//        };
//        let def_item = ModuleDefinitionItem {
//            public: true,
//            attributes: Vec::new(),
//            value: ModuleDefinitionValue::DataType(data_type),
//        };
//        Ok(ItemRef::from_def(def_item))
//    }
}

//impl<'source> IntoStatic for TableDefinition<'source> {
//    type Result = TableDefinition<'static>;
//    fn into_static(self) -> Self::Result {
//        let TableDefinition { name, pos, body } = self;
//        TableDefinition {
//            name: name.into_static(),
//            pos,
//            body: body.into_static(),
//        }
//    }
//}

//impl SemanticResolve for TableDefinition<'static> {
//    fn is_resolved(&self, context: &ResolveContext) -> bool {
//        self.body.is_resolved(context)
//    }
//    fn try_resolve(&mut self, context: &mut ResolveContext) {
//        self.body.try_resolve(context)
//    }
//}

#[derive(Debug, Clone, PartialEq)]
pub struct TableDefinition {
    pub name: String,
    pub pos: ItemPosition,
    pub body: IndexMap<String, Field>,
}

impl TableDefinition {
    #[inline]
    pub fn make_entity_type(&self) -> DataType {
        DataType::Compound(CompoundDataType::Structure(self.body.clone()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalItemTailAST<'source> {
    None,
    Asterisk,
    Alias(Identifier<'source>),
}

impl<'source> Into<ExternalItemTail> for ExternalItemTailAST<'source> {
    fn into(self) -> ExternalItemTail {
        match self {
            ExternalItemTailAST::None => ExternalItemTail::None,
            ExternalItemTailAST::Asterisk => ExternalItemTail::Asterisk,
            ExternalItemTailAST::Alias(ident) => ExternalItemTail::Alias(ident.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalItemTail {
    None,
    Asterisk,
    Alias(String),
}

//impl<'source> IntoStatic for ExternalItemTail<'source> {
//    type Result = ExternalItemTail<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            ExternalItemTail::None => ExternalItemTail::None,
//            ExternalItemTail::Asterisk => ExternalItemTail::Asterisk,
//            ExternalItemTail::Alias(identifier) => ExternalItemTail::Alias(identifier.into_static()),
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalItemImportAST<'source> {
    pub path: ItemPath,
    pub tail: ExternalItemTailAST<'source>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExternalItemImport {
    pub item: SyncRef<Item>,
    pub tail: ExternalItemTail,
}

impl ExternalItemImportAST<'static> {
//    pub fn try_semantic_resolve(&mut self, context: &mut ResolveContext) -> Option<ItemBody> {
//        match context.resolve_item(&self.path) {
//            Ok(item) => {
//                let name = self.path.path.last()
//                    .expect("Path should not be empty")
//                    .clone();
//                match &self.tail {
//                    &ExternalItemTail::None => Some(ItemBody::ImportItem {
//                        name: name.clone(),
//                        original_name: name,
//                        item,
//                    }),
//                    &ExternalItemTail::Alias(ref alias) => Some(ItemBody::ImportItem {
//                        name: alias.clone(),
//                        original_name: name,
//                        item,
//                    }),
//                    &ExternalItemTail::Asterisk => match item.get_module(self.path.pos) {
//                        Ok(module) => Some(ItemBody::ModuleReference { module }),
//                        Err(err) => {
//                            context.throw_error(err);
//                            None
//                        }
//                    },
//                }
//            },
//            Err(err) => {
//                context.throw_error(err);
//                context.request_dependency(self.path.clone());
//                None
//            }
//        }
//    }
//    pub fn try_put_dependency(&self, dependency: &StaticPath, module: &ModuleRef) -> Result<Option<ItemBody>, SemanticError> {
//        let dependency_len = dependency.path.len();
//        println!("Comparing paths (begin of {:?} and {:?}", dependency.path, self.path.path);
//        if (self.path.path.len() >= dependency_len)
//            &&
//            (dependency.path.as_slice() == &self.path.path[..dependency_len]) {
//            println!("Begin of dependency's path is equal to import's path. Trying to find item in dependency.");
//            let item_path = &self.path.path[dependency_len..];
//            match module.find_item(item_path) {
//                Some(item) => {
//                    println!("Item found, putting {:?}", item);
//                    let name = self.path.path.last()
//                        .expect("Path should not be empty!")
//                        .clone();
//                    match &self.tail {
//                        &ExternalItemTail::None => {
//                            Ok(Some(ItemBody::ImportItem {
//                                name: name.clone(),
//                                original_name: name,
//                                item,
//                            }))
//                        }
//                        &ExternalItemTail::Alias(ref alias) => {
//                            Ok(Some(ItemBody::ImportItem {
//                                name: alias.clone(),
//                                original_name: name,
//                                item,
//                            }))
//                        }
//                        &ExternalItemTail::Asterisk => {
//                            match item.get_module(self.path.pos) {
//                                Ok(module) => Ok(Some(ItemBody::ModuleReference { module })),
//                                Err(err) => Err(err),
//                            }
//                        }
//                    }
//                }
//                None => Ok(None),
//            }
//        } else {
//            Ok(None)
//        }
//    }
}

//impl<'source> IntoStatic for ExternalItemImport<'source> {
//    type Result = ExternalItemImport<'static>;
//    fn into_static(self) -> Self::Result {
//        let ExternalItemImport { path, tail } = self;
//        ExternalItemImport {
//            path: path.into_static(),
//            tail: tail.into_static(),
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleDefinitionValueAST<'source> {
    DataType(DataTypeDefinitionAST<'source>),
    Table(TableDefinitionAST<'source>),
    Function(FunctionDefinitionAST<'source>),
    Module(ModuleDefinitionAST<'source>),
    Import(ExternalItemImportAST<'source>),
}

impl<'source> ModuleDefinitionValueAST<'source> {
    pub fn name(&'source self) -> &'source str {
        match self {
            ModuleDefinitionValueAST::DataType(def) => def.name.text(),
            ModuleDefinitionValueAST::Import(def) => {
                match &def.tail {
                    &ExternalItemTailAST::None | &ExternalItemTailAST::Asterisk => {
                        def.path.path.as_path()
                            .pop_right()
                            .expect("Import's path should not be empty!")
                    }
                    &ExternalItemTailAST::Alias(ref alias) => {
                        alias.text()
                    }
                }
            }
            ModuleDefinitionValueAST::Function(def) => def.name.text(),
            ModuleDefinitionValueAST::Table(def) => def.name.text(),
            _ => unimplemented!(),
        }
    }
}

impl<'source> Resolve<(SyncRef<Module>, Vec<AttributeAST<'source>>)> for ModuleDefinitionValueAST<'source> {
    type Result = SyncRef<Item>;
    type Error = SemanticError;
    #[allow(unused_assignments)]
    fn resolve(&self, ctx: &(SyncRef<Module>, Vec<AttributeAST<'source>>)) -> Result<Self::Result, Vec<Self::Error>> {
        match self {
            ModuleDefinitionValueAST::DataType(def) => {
                Ok(SyncRef::new(def.resolve(&ctx.0)?))
            }
            ModuleDefinitionValueAST::Import(
                ExternalItemImportAST { path, tail }
            ) => {
                let mut item_path = path.path.as_path();
                let item = match ctx.0.resolve_import(item_path) {
                    Some(item) => item,
                    None => return SemanticError::unresolved_item(path.pos, path.path.clone()).into_err_vec(),
                };
                if *tail == ExternalItemTailAST::Asterisk {
                    let item = item.read();
                    match item.get_module_ref() {
                        Some(module) => {
                            ctx.0.inject_import_module(module.clone());
                        }
                        None => return SemanticError::expected_item_of_another_type(
                            path.pos,
                            SemanticItemType::Module,
                            item.get_type(),
                        )
                            .into_err_vec(),
                    }
                }
                Ok(item)
            }
            ModuleDefinitionValueAST::Function(def) => {
                let def = def.resolve(ctx)?;
                Ok(SyncRef::new(Item::function(def)))
            }
            ModuleDefinitionValueAST::Table(def) => {
                let def = def.resolve(&ctx.0)?;
                Ok(SyncRef::new(Item::table(def)))
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleDefinitionValue {
    DataType(DataTypeDefinition),
    Table(TableDefinition),
    //    Function(FunctionDefinition),
//    Module(ModuleDefinition),
    Import(ExternalItemImport),
}

//impl<'source> IntoStatic for ModuleDefinitionValue<'source> {
//    type Result = ModuleDefinitionValue<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            ModuleDefinitionValue::DataType(value) => ModuleDefinitionValue::DataType(value.into_static()),
//            ModuleDefinitionValue::Table(value) => ModuleDefinitionValue::Table(value.into_static()),
//            ModuleDefinitionValue::Function(value) => ModuleDefinitionValue::Function(value.into_static()),
//            ModuleDefinitionValue::Module(value) => ModuleDefinitionValue::Module(value.into_static()),
//            ModuleDefinitionValue::Import(value) => ModuleDefinitionValue::Import(value.into_static()),
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDefinitionItemAST<'source> {
    pub public: bool,
    pub position: ItemPosition,
    pub attributes: Vec<AttributeAST<'source>>,
    pub value: ModuleDefinitionValueAST<'source>,
}

impl<'source> Resolve<SyncRef<Module>> for ModuleDefinitionItemAST<'source> {
    type Result = ();
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        let ModuleDefinitionItemAST { ref public, ref position, ref attributes, ref value } = self;
        let item = {
            let ctx = (ctx.clone(), attributes.clone());
            let value = value.resolve(&ctx)?;
            ModuleDefinitionItem {
                public: *public,
                position: *position,
                attributes: ctx.1.iter()
                    .map(|attr| attr.into())
                    .collect(),
                value,
            }
        };
        let name = value.name();
        ctx.put_item(name, item);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDefinitionItem {
    pub public: bool,
    pub position: ItemPosition,
    // TODO Продумать перемещение аттрибутов дефиниции
    pub attributes: Vec<Attribute>,
    pub value: SyncRef<Item>,
}

//impl<'source> IntoStatic for ModuleDefinitionItem<'source> {
//    type Result = ModuleDefinitionItem<'static>;
//    fn into_static(self) -> Self::Result {
//        let ModuleDefinitionItem {
//            public, attributes, value
//        } = self;
//        ModuleDefinitionItem {
//            public,
//            attributes: attributes.into_static(),
//            value: value.into_static(),
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDefinitionAST<'source> {
    pub name: Identifier<'source>,
    pub items: Vec<ModuleDefinitionItemAST<'source>>,
}

//impl<'source> IntoStatic for ModuleDefinition<'source> {
//    type Result = ModuleDefinition<'static>;
//    fn into_static(self) -> Self::Result {
//        let ModuleDefinition { name, items } = self;
//        ModuleDefinition {
//            name: name.into_static(),
//            items: items.into_static(),
//        }
//    }
//}
