use indexmap::IndexMap;
use std::hash::Hash;

use lexeme_scanner::Token;

use parser_basics::{
    ParserErrorKind,
    ParserInput,
    ParserResult,
};

use parser_basics::{
    comma_list,
    identifier,
    symbols,
};

use desc_lang::primitives::primitive_data_type;

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
parser_rule!(attributes(i) -> Vec<Attribute<'source>> {
    many0!(i, attribute)
});

/// attributes identifier ":" data_type
parser_rule!(struct_field(i) -> (&'source str, Field<'source>) {
    do_parse!(i,
        attributes: attributes >>
        name: identifier >>
        apply!(symbols, ":") >>
        field_type: data_type >>
        ((name, Field { attributes, field_type }))
    )
});

/// attributes data_type
parser_rule!(tuple_field(i) -> Field<'source> {
    do_parse!(i,
        attributes: attributes >>
        field_type: data_type >>
        (Field { attributes, field_type })
    )
});

fn slice_to_map<K: Eq + Hash + Clone + ToString, V: Clone>(input: &[(K, V)]) -> Result<IndexMap<K, V>, ParserErrorKind> {
    let mut result = IndexMap::new();
    for &(ref key, ref value) in input {
        if let Some(_) = result.insert(key.clone(), value.clone()) {
            return Err(ParserErrorKind::key_is_not_unique(key.clone()));
        }
    }
    Ok(result)
}

/// attributes "{" ...struct_field "}"
parser_rule!(struct_body(i) -> StructureDataType<'source> {
    do_parse!(i,
        attributes: attributes >>
        apply!(symbols, "{") >>
        fields_vec: apply!(comma_list, struct_field) >>
        apply!(symbols, "}") >>
        ({
            let fields = match slice_to_map(fields_vec.as_slice()) {
                Ok(v) => v,
                Err(kind) => {
                    println!("Found error while parsing struct {:?}", kind);
                    return i.err(kind);
                },
            };
            StructureDataType { attributes, fields }
        })
    )
});

/// attributes "(" ...tuple_field ")"
parser_rule!(tuple_body(i) -> TupleDataType<'source> {
    do_parse!(i,
        attributes: attributes >>
        apply!(symbols, "(") >>
        fields: apply!(comma_list, tuple_field) >>
        apply!(symbols, ")") >>
        (TupleDataType { attributes, fields })
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
        | identifier => { |x| DataType::Reference(x) }
    )
}
