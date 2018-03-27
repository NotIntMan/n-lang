use helpers::into_static::IntoStatic;
use parser_basics::Identifier;
use syntax_parser::compound_types::{
    Attribute,
    DataType,
    Field,
};
use syntax_parser::functions::FunctionDefinition;
use syntax_parser::others::Path;

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
    pub body: Vec<(Identifier<'source>, Field<'source>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalItemTail<'source> {
    None,
    Asterisk,
    Alias(Identifier<'source>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalItemImport<'source> {
    pub path: Path<'source>,
    pub tail: ExternalItemTail<'source>,
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
