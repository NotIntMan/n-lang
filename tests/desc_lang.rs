#[macro_use]
extern crate n_transpiler;
extern crate indexmap;
#[macro_use]
extern crate pretty_assertions;

use n_transpiler::helpers::assertion::Assertion;
use n_transpiler::desc_lang::primitives::*;
use n_transpiler::desc_lang::compounds::*;
use n_transpiler::desc_lang::functions::*;
use n_transpiler::desc_lang::modules::*;
use n_transpiler::man_lang::statements::*;
use n_transpiler::man_lang::expressions::*;

#[test]
fn simple_type_parses_correctly() {
    let result = parse!("big integer", primitive_data_type);
    assert_eq!(
        result,
        PrimitiveDataType::Number(NumberType::Integer {
            zerofill: false,
            unsigned: false,
            integer_type: IntegerType::Big,
        })
    )
}

#[test]
fn simple_type_with_size_parses_correctly() {
    let result = parse!("unsigned decimal(2)", primitive_data_type);
    assert_eq!(
        result,
        PrimitiveDataType::Number(NumberType::Decimal {
            zerofill: false,
            unsigned: true,
            size: Some((2, None)),
        })
    )
}

#[test]
fn simple_type_with_complex_size_parses_correctly() {
    let result = parse!("double(5, 3)", primitive_data_type);
    assert_eq!(
        result,
        PrimitiveDataType::Number(NumberType::Float {
            double: true,
            size: Some((5, 3)),
        })
    )
}

#[test]
fn simple_year_type_parses_correctly() {
    let result = parse!("year", primitive_data_type);
    assert_eq!(
        result,
        PrimitiveDataType::Year(YearType::Year4)
    )
}

#[test]
fn simple_datetime_type_parses_correctly() {
    let result = parse!("datetime(4)", primitive_data_type);
    assert_eq!(
        result,
        PrimitiveDataType::DateTime(DateTimeType::Datetime {
            precision: Some(4),
        })
    )
}

#[test]
fn simple_varchar_type_with_encoding_parses_correctly() {
    let result = parse!("varchar(256) character set binary", primitive_data_type);
    assert_eq!(
        result,
        PrimitiveDataType::String(StringType::Varchar {
            size: Some(256),
            character_set: Some(CharacterSet::Binary),
        })
    )
}

#[test]
fn simple_varchar_type_parses_correctly() {
    let result = parse!("varchar(50)", primitive_data_type);
    assert_eq!(
        result,
        PrimitiveDataType::String(StringType::Varchar {
            size: Some(50),
            character_set: None,
        })
    )
}

#[test]
fn simple_text_type_parses_correctly() {
    let result = parse!("text character set utf8", primitive_data_type);
    assert_eq!(
        result,
        PrimitiveDataType::String(StringType::Text {
            character_set: Some(CharacterSet::UTF8)
        })
    )
}

#[test]
fn struct_and_tuple_bodies_parses_correctly() {
    let result = parse!("(boolean, {a: integer, b: double})", data_type);
    assert_eq!(result, DataType::Compound(CompoundDataType::Tuple(vec![
            Field {
                attributes: vec![],
                field_type: DataType::Primitive(PrimitiveDataType::Number(NumberType::Boolean)),
            },
            Field {
                attributes: vec![],
                field_type: DataType::Compound(CompoundDataType::Structure(vec![
                    ("a", Field {
                        attributes: vec![],
                        field_type: DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
                            integer_type: IntegerType::Normal,
                            unsigned: false,
                            zerofill: false,
                        })),
                    }),
                    ("b", Field {
                        attributes: vec![],
                        field_type: DataType::Primitive(PrimitiveDataType::Number(NumberType::Float {
                            size: None,
                            double: true,
                        })),
                    }),
                ])),
            },
        ],
    )));
}

#[test]
fn simple_external_function_parses_correctly() {
    let result: FunctionDefinition = parse!("extern fn sum(a: integer, b: integer): big integer", function_definition);
    assert_eq!(result.name, "sum");
    let (arg_name, arg_type) = result.arguments.get_index(0)
        .expect("Function's arguments must have the first item");
    assert_eq!(*arg_name, "a");
    arg_type.assert("integer");
    let (arg_name, arg_type) = result.arguments.get_index(1)
        .expect("Function's arguments must have the second item");
    assert_eq!(*arg_name, "b");
    arg_type.assert("integer");
    assert_eq!(result.arguments.get_index(2), None);
    result.result.assert(&Some("big integer"));
    assert_eq!(result.body, FunctionBody::External);
}

#[test]
fn simple_const_time_function_parses_correctly() {
    let result: FunctionDefinition = parse!("\
            fn sum_of_k_series_of_n (k: unsigned integer): unsigned big integer {
                let a := k / 2;
                let b: big integer := k + 1;
                b := a * b;
                return b;
            }
        ", function_definition);
    assert_eq!(result.name, "sum_of_k_series_of_n");
    let (arg_name, arg_type) = result.arguments.get_index(0)
        .expect("Function's arguments must have the first item");
    assert_eq!(*arg_name, "k");
    assert_eq!(*arg_type, DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
        integer_type: IntegerType::Normal,
        unsigned: true,
        zerofill: false,
    })));
    assert_eq!(result.arguments.get_index(1), None);
    assert_eq!(result.result, Some(DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
        integer_type: IntegerType::Big,
        unsigned: true,
        zerofill: false,
    }))));
    let mut statement_iterator = match result.body {
        FunctionBody::Implementation(statement) => match statement {
            Statement::Block { statements } => statements.into_iter(),
            o => panic!("Pattern FunctionBody::Implementation do not matches this value: {:?}", o),
        },
        o => panic!("Pattern FunctionBody::Implementation do not matches this value: {:?}", o),
    };
    let statement = statement_iterator.next()
        .expect("Function's body must have the first statement");
    match_it!(statement, Statement::VariableDefinition { name, data_type, default_value } => {
            assert_eq!(name, "a");
            assert_eq!(data_type, None);
            match_it!(default_value, Some(Expression::BinaryOperation(left, op, right)) => {
                assert_eq!(op, BinaryOperator::Divide);
                left.assert("k");
                right.assert("2");
            });
        });
    let statement = statement_iterator.next()
        .expect("Function's body must have the second statement");
    match_it!(statement, Statement::VariableDefinition { name, data_type, default_value } => {
            assert_eq!(name, "b");
            assert_eq!(data_type, Some(DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
                integer_type: IntegerType::Big,
                unsigned: false,
                zerofill: false,
            }))));
            match_it!(default_value, Some(Expression::BinaryOperation(left, op, right)) => {
                assert_eq!(op, BinaryOperator::Plus);
                left.assert("k");
                right.assert("1");
            });
        });
    let statement = statement_iterator.next()
        .expect("Function's body must have the second statement");
    match_it!(statement, Statement::VariableAssignment { name, expression } => {
            assert_eq!(name, "b");
            match_it!(expression, Expression::BinaryOperation(left, op, right) => {
                assert_eq!(op, BinaryOperator::Times);
                left.assert("a");
                right.assert("b");
            });
        });
    let statement = statement_iterator.next()
        .expect("Function's body must have the second statement");
    match_it!(statement, Statement::Return { value } => {
            value.assert(&Some("b"));
        });
    assert_eq!(statement_iterator.next(), None);
}

#[test]
fn module_of_two_usage_parses_correctly() {
    let result: Vec<ModuleDefinitionItem> = parse!("\
        use foo::bar as Bar;
        #[no_mandle]
        pub use foo::TakeAll;
    ", module);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].public, false);
    assert_eq!(result[0].attributes.len(), 0);
    match_it!(&result[0].value, &ModuleDefinitionValue::Import(ExternalItemImport { ref path, alias }) => {
        assert_eq!(*path, ["foo", "bar"]);
        assert_eq!(alias, Some("Bar"));
    });
    assert_eq!(result[1].public, true);
    assert_eq!(result[1].attributes.len(), 1);
    assert_eq!(result[1].attributes[0].name, "no_mandle");
    assert_eq!(result[1].attributes[0].arguments, None);
    match_it!(&result[1].value, &ModuleDefinitionValue::Import(ExternalItemImport { ref path, alias }) => {
        assert_eq!(*path, ["foo", "TakeAll"]);
        assert_eq!(alias, None);
    });
}

#[test]
fn module_of_table_and_struct_parses_correctly() {
    let result: Vec<ModuleDefinitionItem> = parse!("\
        #[derive(Hash)]
        pub struct Complex {
            real: double,
            imag: float,
        }

        pub table Signals {
            #[primary_key]
            #[auto_increment]
            id: unsigned integer,
            #[check(A, B)]
            #[check_fn(X, YY)]
            value: Complex,
        }
    ", module);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].public, true);
    assert_eq!(result[0].attributes.len(), 1);
    assert_eq!(result[0].attributes[0].name, "derive");
    assert_eq!(result[0].attributes[0].arguments, Some(vec!["Hash"]));
    match_it!(&result[0].value, &ModuleDefinitionValue::DataType(DataTypeDefinition { name, ref body }) => {
        assert_eq!(name, "Complex");
        body.assert("{ real: double, imag: float }");
    });
    assert_eq!(result[1].public, true);
    assert_eq!(result[1].attributes.len(), 0);
    match_it!(&result[1].value, &ModuleDefinitionValue::Table(TableDefinition { name, ref body }) => {
        assert_eq!(name, "Signals");
        let mut body_iter = body.iter();
        match_it!(body_iter.next(), Some(&("id", ref field)) => {
            assert_eq!(field.attributes.len(), 2);
            assert_eq!(field.attributes[0].name, "primary_key");
            assert_eq!(field.attributes[0].arguments, None);
            assert_eq!(field.attributes[1].name, "auto_increment");
            assert_eq!(field.attributes[1].arguments, None);
            field.field_type.assert("unsigned integer");
        });
        match_it!(body_iter.next(), Some(&("value", ref field)) => {
            assert_eq!(field.attributes.len(), 2);
            assert_eq!(field.attributes[0].name, "check");
            assert_eq!(field.attributes[0].arguments, Some(vec!["A", "B"]));
            assert_eq!(field.attributes[1].name, "check_fn");
            assert_eq!(field.attributes[1].arguments, Some(vec!["X", "YY"]));
            field.field_type.assert("Complex");
        });
        assert_eq!(body_iter.next(), None);
    });
}
