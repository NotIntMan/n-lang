extern crate n_transpiler;
extern crate indexmap;

use indexmap::IndexMap;

use n_transpiler::lexeme_scanner::Scanner;
use n_transpiler::parser_basics::parse;
use n_transpiler::desc_lang::primitives::*;
use n_transpiler::desc_lang::compounds::*;

#[test]
fn simple_type_parses_correctly() {
    let tokens = Scanner::scan("big integer")
        .expect("Scanner result must be ok");
    assert_eq!(
        parse(tokens.as_slice(), primitive_data_type)
            .expect("Parser result must be ok"),
        PrimitiveDataType::Number(NumberType::Integer {
            zerofill: false,
            unsigned: false,
            integer_type: IntegerType::Big,
        })
    )
}

#[test]
fn simple_type_with_size_parses_correctly() {
    let tokens = Scanner::scan("unsigned decimal (2)")
        .expect("Scanner result must be ok");
    assert_eq!(
        parse(tokens.as_slice(), primitive_data_type)
            .expect("Parser result must be ok"),
        PrimitiveDataType::Number(NumberType::Decimal {
            zerofill: false,
            unsigned: true,
            size: Some((2, None)),
        })
    )
}

#[test]
fn simple_type_with_complex_size_parses_correctly() {
    let tokens = Scanner::scan("double(5,3)")
        .expect("Scanner result must be ok");
    assert_eq!(
        parse(tokens.as_slice(), primitive_data_type)
            .expect("Parser result must be ok"),
        PrimitiveDataType::Number(NumberType::Float {
            double: true,
            size: Some((5, 3)),
        })
    )
}

#[test]
fn simple_year_type_parses_correctly() {
    let tokens = Scanner::scan("year")
        .expect("Scanner result must be ok");
    assert_eq!(
        parse(tokens.as_slice(), primitive_data_type)
            .expect("Parser result must be ok"),
        PrimitiveDataType::Year(YearType::Year4)
    )
}

#[test]
fn simple_datetime_type_parses_correctly() {
    let tokens = Scanner::scan("datetime(4)")
        .expect("Scanner result must be ok");
    assert_eq!(
        parse(tokens.as_slice(), primitive_data_type)
            .expect("Parser result must be ok"),
        PrimitiveDataType::DateTime(DateTimeType::Datetime {
            precision: Some(4),
        })
    )
}

#[test]
fn simple_varchar_type_with_encoding_parses_correctly() {
    let tokens = Scanner::scan("varchar(256) character set binary")
        .expect("Scanner result must be ok");
    assert_eq!(
        parse(tokens.as_slice(), primitive_data_type)
            .expect("Parser result must be ok"),
        PrimitiveDataType::String(StringType::Varchar {
            size: Some(256),
            character_set: Some(CharacterSet::Binary),
        })
    )
}

#[test]
fn simple_varchar_type_parses_correctly() {
    let tokens = Scanner::scan("varchar(50)")
        .expect("Scanner result must be ok");
    assert_eq!(
        parse(tokens.as_slice(), primitive_data_type)
            .expect("Parser result must be ok"),
        PrimitiveDataType::String(StringType::Varchar {
            size: Some(50),
            character_set: None,
        })
    )
}

#[test]
fn simple_text_type_parses_correctly() {
    let tokens = Scanner::scan("text character set utf8")
        .expect("Scanner result must be ok");
    assert_eq!(
        parse(tokens.as_slice(), primitive_data_type)
            .expect("Parser result must be ok"),
        PrimitiveDataType::String(StringType::Text {
            character_set: Some(CharacterSet::UTF8)
        })
    )
}

#[test]
fn struct_and_tuple_bodies_parses_correctly() {
    let tokens = Scanner::scan("(boolean, {a: integer, b: double})")
        .expect("Scanner result must be ok");
    let result = parse(tokens.as_slice(), data_type)
        .expect("Parser result must be ok");
    assert_eq!(result, DataType::Compound(CompoundDataType::Tuple(TupleDataType {
        attributes: vec![],
        fields: vec![
            Field {
                attributes: vec![],
                field_type: DataType::Primitive(PrimitiveDataType::Number(NumberType::Boolean)),
            },
            Field {
                attributes: vec![],
                field_type: DataType::Compound(CompoundDataType::Structure(StructureDataType {
                    attributes: vec![],
                    fields: {
                        let mut map = IndexMap::new();
                        map.insert("a", Field {
                            attributes: vec![],
                            field_type: DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
                                integer_type: IntegerType::Normal,
                                unsigned: false,
                                zerofill: false,
                            })),
                        });
                        map.insert("b", Field {
                            attributes: vec![],
                            field_type: DataType::Primitive(PrimitiveDataType::Number(NumberType::Float {
                                size: None,
                                double: true,
                            })),
                        });
                        map
                    },
                })),
            },
        ],
    })));
}
