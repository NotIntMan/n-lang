use parser_basics::Identifier;
use syntax_parser::compound_types::DataType;
use syntax_parser::statements::Statement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionBody<'source> {
    External,
    Implementation(Statement<'source>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition<'source> {
    pub name: Identifier<'source>,
    pub arguments: Vec<(Identifier<'source>, DataType<'source>)>,
    pub result: Option<DataType<'source>>,
    pub body: FunctionBody<'source>,
}
