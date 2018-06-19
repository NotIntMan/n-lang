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
use language::{
    AttributeAST,
    CompoundDataTypeAST,
    DataTypeAST,
    FieldAST,
    primitive_data_type,
    module_path,
};

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

/// Парсер, реализующий разбор грамматики составных типов
pub fn compound_type<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, DataTypeAST<'source>> {
    alt!(input,
        struct_body => { |x| DataTypeAST::Compound(CompoundDataTypeAST::Structure(x)) }
        | tuple_body => { |x| DataTypeAST::Compound(CompoundDataTypeAST::Tuple(x)) }
    )
}

/// Парсер, реализующий разбор грамматики составных и простых типов
pub fn data_type<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, DataTypeAST<'source>> {
    alt!(input,
        compound_type
        | primitive_data_type => { |x| DataTypeAST::Primitive(x) }
        | module_path => { |x| DataTypeAST::Reference(x) }
    )
}
