use syntax_parser::compound_types::{
    Attribute,
    DataType,
    Field,
};
use syntax_parser::functions::FunctionDefinition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTypeDefinition<'source> {
    pub name: &'source str,
    pub body: DataType<'source>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDefinition<'source> {
    pub name: &'source str,
    pub body: Vec<(&'source str, Field<'source>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalItemImport<'source> {
    pub path: Vec<&'source str>,
    pub alias: Option<&'source str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleDefinitionValue<'source> {
    DataType(DataTypeDefinition<'source>),
    Table(TableDefinition<'source>),
    Function(FunctionDefinition<'source>),
    Module(ModuleDefinition<'source>),
    Import(ExternalItemImport<'source>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDefinitionItem<'source> {
    pub public: bool,
    pub attributes: Vec<Attribute<'source>>,
    pub value: ModuleDefinitionValue<'source>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDefinition<'source> {
    pub name: &'source str,
    pub items: Vec<ModuleDefinitionItem<'source>>,
}
