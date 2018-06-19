extern crate n_lang;

use std::path::Path;
use n_lang::{
    helpers::Resolve,
    helpers::SyncRef,
    language::{
        BinaryOperator,
        DataType,
        PrimitiveDataType,
        NumberType,
    },
    project_analysis::{
        ProjectContext,
        HashMapSource,
        StdLib,
        StdLibBinaryOperation,
        StdLibFunction,
    },
};

#[test]
fn dir_resolve() {
    let mut stdlib = StdLib::new();

    let tiny_unsigned_integer = DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
        unsigned: true,
        zerofill: false,
        size: 8,
    }));

    let small_integer = DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
        unsigned: false,
        zerofill: false,
        size: 16,
    }));

    let boolean = DataType::Primitive(PrimitiveDataType::Number(NumberType::Boolean));

    stdlib.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Plus,
        left: tiny_unsigned_integer.clone(),
        right: tiny_unsigned_integer.clone(),
        output: tiny_unsigned_integer.clone(),
    });

    stdlib.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Plus,
        left: small_integer.clone(),
        right: small_integer.clone(),
        output: small_integer.clone(),
    });

    stdlib.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::MoreThan,
        left: small_integer.clone(),
        right: small_integer.clone(),
        output: boolean.clone(),
    });

    stdlib.reg_function(
        StdLibFunction::new("max".to_string())
            .gets(vec![small_integer.clone()])
            .returns(small_integer.clone())
            .aggregate()
    );

    stdlib.reg_function(
        StdLibFunction::new("sum".to_string())
            .gets(vec![small_integer.clone()])
            .returns(small_integer.clone())
            .aggregate()
    );

    stdlib.reg_function(
        StdLibFunction::new("abs".to_string())
            .gets(vec![small_integer.clone()])
            .returns(small_integer.clone())
            .lite_weight()
    );

    let source = HashMapSource::for_dir(Path::new("./tests/dir_resolve"))
        .expect("Cannot process \"dir_resolve\".");

    let project = ProjectContext::new(SyncRef::new(stdlib));
    for (module_path, _) in source.texts() {
        project.request_resolving_module(module_path.as_path());
    }
    let result = project.resolve(&source);
    if let Err(errors) = &result {
        println!("Got errors:");
        for error in errors {
            println!("{}", error);
        }
    }
    assert!(result.is_ok())
}
