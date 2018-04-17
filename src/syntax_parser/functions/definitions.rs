use helpers::into_static::IntoStatic;
use parser_basics::Identifier;
use syntax_parser::compound_types::DataType;
use syntax_parser::statements::Statement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionBody<'source> {
    External,
    Implementation(Statement<'source>),
}

impl<'source> IntoStatic for FunctionBody<'source> {
    type Result = FunctionBody<'static>;
    fn into_static(self) -> Self::Result {
        match self {
            FunctionBody::External => FunctionBody::External,
            FunctionBody::Implementation(stmt) => FunctionBody::Implementation(stmt.into_static()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition<'source> {
    pub name: Identifier<'source>,
    pub arguments: Vec<(Identifier<'source>, DataType<'source>)>,
    pub result: Option<DataType<'source>>,
    pub body: FunctionBody<'source>,
}

impl<'source> IntoStatic for FunctionDefinition<'source> {
    type Result = FunctionDefinition<'static>;
    fn into_static(self) -> Self::Result {
        let FunctionDefinition { name, arguments, result, body } = self;
        FunctionDefinition {
            name: name.into_static(),
            arguments: arguments.into_static(),
            result: result.into_static(),
            body: body.into_static(),
        }
    }
}
