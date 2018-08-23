use language::{
    AttributeAST,
    CompoundDataTypeAST,
    DataTypeAST,
    DataTypeASTBody,
    FieldAST,
    module_path,
    primitive_data_type,
};
use lexeme_scanner::Token;
use parser_basics::{
    comma_list,
    identifier,
    Identifier,
    item_position,
    symbol_position,
    symbols,
};
use parser_basics::ParserResult;

/// "#[" identifier [(...identifier)] "]"
parser_rule!(attribute(i) -> AttributeAST<'source> {
    do_parse!(i,
        apply!(symbols, "#[") >>
        name: identifier >>
        arguments: opt!(do_parse!(
            apply!(symbols, "(") >>
            x: apply!(comma_list, identifier) >>
            apply!(symbols, ")") >>
            (x)
        )) >>
        apply!(symbols, "]") >>
        (AttributeAST { name, arguments })
    )
});

/// ...attribute
parser_rule!(pub attributes(i) -> Vec<AttributeAST<'source>> {
    many0!(i, attribute)
});

/// attributes identifier ":" data_type
parser_rule!(struct_field(i) -> (Identifier<'source>, FieldAST<'source>) {
    do_parse!(i,
        begin: symbol_position >>
        attributes: attributes >>
        name: identifier >>
        apply!(symbols, ":") >>
        field_type: data_type >>
        position: apply!(item_position, begin) >>
        ((name, FieldAST { attributes, field_type, position }))
    )
});

/// attributes data_type
parser_rule!(tuple_field(i) -> FieldAST<'source> {
    do_parse!(i,
        begin: symbol_position >>
        attributes: attributes >>
        field_type: data_type >>
        position: apply!(item_position, begin) >>
        (FieldAST { attributes, field_type, position })
    )
});

/// attributes "{" ...struct_field "}"
parser_rule!(pub struct_body(i) -> Vec<(Identifier<'source>, FieldAST<'source>)> {
    do_parse!(i,
        apply!(symbols, "{") >>
        fields: apply!(comma_list, struct_field) >>
        apply!(symbols, "}") >>
        (fields)
    )
});

/// attributes "(" ...tuple_field ")"
parser_rule!(tuple_body(i) -> Vec<FieldAST<'source>> {
    do_parse!(i,
        apply!(symbols, "(") >>
        fields: apply!(comma_list, tuple_field) >>
        apply!(symbols, ")") >>
        (fields)
    )
});

parser_rule!(compound_body(i) -> CompoundDataTypeAST<'source> {
    alt!(i,
        struct_body => { |x| CompoundDataTypeAST::Structure(x) }
        | tuple_body => { |x| CompoundDataTypeAST::Tuple(x) }
    )
});

/// Парсер, реализующий разбор грамматики составных типов
pub fn compound_type<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, DataTypeAST<'source>> {
    do_parse!(input,
        begin: symbol_position >>
        body: compound_body >>
        pos: apply!(item_position, begin) >>
        (DataTypeAST { pos, body: DataTypeASTBody::Compound(body) })
    )
}

/// Парсер, реализующий разбор грамматики составных и простых типов
pub fn data_type<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, DataTypeAST<'source>> {
    do_parse!(input,
        begin: symbol_position >>
        body: alt!(
            compound_body => { |x| DataTypeASTBody::Compound(x) }
            | primitive_data_type => { |x| DataTypeASTBody::Primitive(x) }
            | module_path => { |x| DataTypeASTBody::Reference(x) }
        ) >>
        pos: apply!(item_position, begin) >>
        (DataTypeAST { pos, body })
    )
}
