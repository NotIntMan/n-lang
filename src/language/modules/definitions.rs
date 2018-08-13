use helpers::{
    as_unique_identifier,
    BlockFormatter,
    Extractor,
    Generate,
    PathBuf,
    Resolve,
    SyncRef,
    TSQL,
    TSQLParameters,
};
use indexmap::IndexMap;
use language::{
    Attribute,
    AttributeAST,
    CompoundDataType,
    DataType,
    DataTypeAST,
    Field,
    FieldAST,
    FieldPrimitive,
    find_attribute,
    FunctionDefinitionAST,
    ItemPath,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use project_analysis::{
    Item,
    Module,
    SemanticError,
    SemanticItemType,
};
use std::{
    fmt,
    sync::Arc,
};

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
        Ok(Item::data_type(ctx.clone(), def))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataTypeDefinition {
    pub name: String,
    pub body: DataType,
}

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
            Ok(map) => Arc::new(map.resolve(ctx)?),
            Err(name) => return SemanticError::duplicate_definition(
                name.item_pos(),
                name.text().to_string(),
                SemanticItemType::Field,
            )
                .into_err_vec(),
        };
        let entity = DataType::Compound(CompoundDataType::Structure(body.clone()));
        let primary_key = {
            let mut primary_key = IndexMap::new();
            for (name, field) in body.iter() {
                let is_primary_key_part = find_attribute(
                    field.attributes.as_slice(),
                    "primary_key",
                ).is_some();
                if is_primary_key_part {
                    primary_key.insert(name.clone(), field.clone());
                }
            }
            DataType::Compound(CompoundDataType::Structure(Arc::new(primary_key)))
        };
        Ok(TableDefinition {
            name: self.name.to_string(),
            pos: self.pos,
            body,
            entity,
            primary_key,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableDefinition {
    pub name: String,
    pub pos: ItemPosition,
    pub body: Arc<IndexMap<String, Field>>,
    pub entity: DataType,
    pub primary_key: DataType,
}

impl TableDefinition {
    pub fn fmt_primitives_as_columns(
        mut f: BlockFormatter<impl fmt::Write>,
        parameters: TSQLParameters,
        columns: impl IntoIterator<Item=FieldPrimitive>,
        last_comma: bool,
        postfix: Option<&str>,
    ) -> fmt::Result {
        let mut columns = columns.into_iter().peekable();
        while let Some(primitive) = columns.next() {
            let mut line = f.line()?;
            line.write(format_args!("[{}] {}", primitive.path, TSQL(&primitive.field_type, parameters.clone())))?;
            if let Some(postfix) = &postfix {
                line.write(format_args!(" {}", postfix))?;
            }
            if last_comma || columns.peek().is_some() {
                line.write(",")?;
            }
        }
        Ok(())
    }
}

impl<'a> Generate<TSQLParameters<'a>> for TableDefinition {
    fn fmt(&self, mut root: BlockFormatter<impl fmt::Write>, parameters: TSQLParameters<'a>) -> fmt::Result {
        {
            let mut line = root.line()?;
            line.write("CREATE TABLE [")?;
            if !parameters.module_path.data.is_empty() {
                line.write(format_args!("{}{}", parameters.module_path.data, parameters.module_path.delimiter))?;
            }
            line.write(format_args!("{}] (", self.name))?;
        }

        let mut columns = root.sub_block();

        let mut primitives = Vec::new();
        for (field_name, field) in self.body.iter() {
            let mut prefix = PathBuf::new("#");
            prefix.push(field_name.as_str());
            let modifier = find_attribute(&field.attributes, "auto_increment")
                .map(|_| "IDENTITY");
            field.field_type.make_primitives(prefix, &mut primitives);
            TableDefinition::fmt_primitives_as_columns(
                columns.clone(),
                parameters.clone(),
                Extractor::new(&mut primitives),
                true,
                modifier,
            )?;
        }

        {
            let mut primary_key = columns.line()?;
            primary_key.write("PRIMARY KEY (")?;
            self.primary_key.make_primitives(PathBuf::new("#"), &mut primitives);

            let mut primitives = Extractor::new(&mut primitives);
            if let Some(primitive) = primitives.next() {
                primary_key.write(format_args!("[{}]", primitive.path.data))?;
            }
            for primitive in primitives {
                primary_key.write(format_args!(", {}", primitive.path.data))?;
            }
            primary_key.write(")")?;
        }

        root.write_line(")")
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
            ExternalItemTailAST::Alias(identifier) => ExternalItemTail::Alias(identifier.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalItemTail {
    None,
    Asterisk,
    Alias(String),
}

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
                    ExternalItemTailAST::None | &ExternalItemTailAST::Asterisk => {
                        def.path.path.as_path()
                            .pop_right()
                            .expect("Import's path should not be empty!")
                    }
                    ExternalItemTailAST::Alias(alias) => {
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

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleDefinitionValue {
    DataType(DataTypeDefinition),
    Table(TableDefinition),
    //    Function(FunctionDefinition),
//    Module(ModuleDefinition),
    Import(ExternalItemImport),
}

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
        let ModuleDefinitionItemAST { public, position, attributes, value } = self;
        let item = {
            let value = match value {
                ModuleDefinitionValueAST::DataType(def) => {
                    SyncRef::new(def.resolve(ctx)?)
                }
                ModuleDefinitionValueAST::Import(
                    ExternalItemImportAST { path, tail }
                ) => {
                    let mut item_path = path.path.as_path();
                    let item = match ctx.resolve_import(item_path) {
                        Some(item) => item,
                        None => return SemanticError::unresolved_item(path.pos, path.path.clone()).into_err_vec(),
                    };
                    if *tail == ExternalItemTailAST::Asterisk {
                        let item = item.read();
                        match item.get_module_ref() {
                            Some(module) => {
                                ctx.inject_import_module(module.clone());
                            }
                            None => return SemanticError::expected_item_of_another_type(
                                path.pos,
                                SemanticItemType::Module,
                                item.get_type(),
                            )
                                .into_err_vec(),
                        }
                    }
                    item
                }
                ModuleDefinitionValueAST::Function(def) => {
                    let ctx = (ctx.clone(), attributes.clone());
                    let def = def.resolve(&ctx)?;
                    SyncRef::new(Item::function(ctx.0.clone(), def))
                }
                ModuleDefinitionValueAST::Table(def) => {
                    let def = def.resolve(ctx)?;
                    SyncRef::new(Item::table(ctx.clone(), def))
                }
                ModuleDefinitionValueAST::Module(_) => {
                    return SemanticError::not_supported_yet(self.position, "file-scoped modules")
                        .into_err_vec();
                }
            };
            ModuleDefinitionItem {
                public: *public,
                position: *position,
                attributes: attributes.iter()
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

#[derive(Debug, Clone, PartialEq)]
pub struct ModuleDefinitionAST<'source> {
    pub name: Identifier<'source>,
    pub items: Vec<ModuleDefinitionItemAST<'source>>,
}
