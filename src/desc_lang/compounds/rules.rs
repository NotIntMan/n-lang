use std::collections::HashMap;

use parser_basics::{
    comma_list,
    identifier,
    rounded_comma_list,
    symbols,
};

use desc_lang::primitives::primitive_data_type;

use super::*;

// TODO Понять почему тут всё ломается
parser_rule!(attribute(i) -> Attribute<'source> {
    do_parse!(i,
        apply!(symbols, "#[") >>
        name: identifier >>
        arguments: prepare!(rounded_comma_list(identifier)) >>
        apply!(symbols, "]") >>
        (Attribute { name, arguments })
    )
});

parser_rule!(attributes(i) -> Option<Vec<Attribute<'source>>> {
    opt!(i, many1!(attribute))
});

// TODO Поправить field_type так, чтобы он ссылался на корневое правило разбора типа данных
parser_rule!(struct_field(i) -> (&'source str, Field<'source>) {
    do_parse!(i,
        attributes: attributes >>
        name: identifier >>
        apply!(symbols, ":") >>
        field_type: identifier >>
        ((name, Field { attributes, field_type }))
    )
});

// TODO Поправить field_type так, чтобы он ссылался на корневое правило разбора типа данных
parser_rule!(tuple_field(i) -> Field<'source> {
    do_parse!(i,
        attributes: opt!(many1!(attribute)) >>
        field_type: identifier >>
        (Field { attributes, field_type })
    )
});

parser_rule!(struct_body(i) -> StructureDataType<'source> {
    do_parse!(i,
        attributes: attributes >>
        apply!(symbols, "{") >>
        fields_vec: prepare!(comma_list(struct_field)) >>
        apply!(symbols, "}") >>
        ({
            let fields = HashMap::from_iter(fields_vec);
            StructureDataType { attributes, fields }
        })
    )
});

parser_rule!(tuple_body(i) -> StructureDataType<'source> {
    do_parse!(i,
        attributes: attributes >>
        apply!(symbols, "(") >>
        fields_vec: prepare!(comma_list(tuple_field)) >>
        apply!(symbols, ")") >>
        (TupleDataType { attributes, fields })
    )
});


/// Парсер, реализующий разбор грамматики составных типов
pub fn compound_data_type<'a, 'b>(input: &'a [Token<'b>]) -> ParserResult<'a, 'b, PrimitiveDataType> {
    alt!(input,
        primitive_data_type => { |x| DataType::Primitive(x) } |
        struct_body => { |x| DataType::Structure(x) } |
        tuple_body => { |x| DataType::Tuple(x) } |
        identifier => { |x| DataType::Reference(x) }
    )
}

#[test]
fn x0() {
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
    let tokens = Scanner::scan("(boolean, {a: integer, b: double})")
        .expect("Scanner result must be ok");
    let result = parse(tokens.as_slice(), compound_data_type)
        .expect("Parser result must be ok");
    println!("{:?}", result);
}
