use lexeme_scanner::Token;
use parser_basics::ParserResult;
use parser_basics::{
    comma_list,
    identifier,
    Identifier,
    symbols,
    item_position,
    symbol_position,
};

use syntax_parser::primitive_types::primitive_data_type;
use syntax_parser::others::module_path;

use super::*;

/// "#[" identifier [(...identifier)] "]"
parser_rule!(attribute(i) -> Attribute<'source> {
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
        (Attribute { name, arguments })
    )
});

/// ...attribute
parser_rule!(pub attributes(i) -> Vec<Attribute<'source>> {
    many0!(i, attribute)
});

/// attributes identifier ":" data_type
parser_rule!(struct_field(i) -> (Identifier<'source>, Field<'source>) {
    do_parse!(i,
        begin: symbol_position >>
        attributes: attributes >>
        name: identifier >>
        apply!(symbols, ":") >>
        field_type: data_type >>
        position: apply!(item_position, begin) >>
        ((name, Field { attributes, field_type, position }))
    )
});

/// attributes data_type
parser_rule!(tuple_field(i) -> Field<'source> {
    do_parse!(i,
        begin: symbol_position >>
        attributes: attributes >>
        field_type: data_type >>
        position: apply!(item_position, begin) >>
        (Field { attributes, field_type, position })
    )
});

/// attributes "{" ...struct_field "}"
parser_rule!(pub struct_body(i) -> Vec<(Identifier<'source>, Field<'source>)> {
    do_parse!(i,
        apply!(symbols, "{") >>
        fields: apply!(comma_list, struct_field) >>
        apply!(symbols, "}") >>
        (fields)
    )
});

/// attributes "(" ...tuple_field ")"
parser_rule!(tuple_body(i) -> Vec<Field<'source>> {
    do_parse!(i,
        apply!(symbols, "(") >>
        fields: apply!(comma_list, tuple_field) >>
        apply!(symbols, ")") >>
        (fields)
    )
});

/// Парсер, реализующий разбор грамматики составных типов
pub fn compound_type<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, DataType<'source>> {
    alt!(input,
        struct_body => { |x| DataType::Compound(CompoundDataType::Structure(x)) }
        | tuple_body => { |x| DataType::Compound(CompoundDataType::Tuple(x)) }
    )
}

/// Парсер, реализующий разбор грамматики составных и простых типов
pub fn data_type<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, DataType<'source>> {
    alt!(input,
        compound_type
        | primitive_data_type => { |x| DataType::Primitive(x) }
        | module_path => { |x| DataType::Reference(x) }
    )
}
