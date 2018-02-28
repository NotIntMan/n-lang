use indexmap::IndexMap;
use desc_lang::compounds::DataType;
use man_lang::statements::Statement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionBody<'source> {
    External,
    Implementation(Statement<'source>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition<'source> {
    pub name: &'source str,
    pub arguments: IndexMap<&'source str, DataType<'source>>,
    pub result: Option<DataType<'source>>,
    pub body: FunctionBody<'source>,
}
