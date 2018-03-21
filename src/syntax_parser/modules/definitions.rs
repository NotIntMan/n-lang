use parser_basics::Identifier;
use syntax_parser::compound_types::{
    Attribute,
    DataType,
    Field,
};
use syntax_parser::functions::FunctionDefinition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTypeDefinition<'source> {
    pub name: Identifier<'source>,
    pub body: DataType<'source>,
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
    pub path: Vec<Identifier<'source>>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDefinitionItem<'source> {
    pub public: bool,
    pub attributes: Vec<Attribute<'source>>,
    pub value: ModuleDefinitionValue<'source>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDefinition<'source> {
    pub name: Identifier<'source>,
    pub items: Vec<ModuleDefinitionItem<'source>>,
}
