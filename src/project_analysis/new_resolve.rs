#[test]
fn do_it() {
    use helpers::Resolve;
    use helpers::{
        Path,
        SyncRef,
    };
    use language::{
        BinaryOperator,
        DataType,
        PrimitiveDataType,
        NumberType,
    };
    use project_analysis::{
        ProjectContext,
        HashMapSource,
        StdLib,
        StdLibBinaryOperation,
        StdLibFunction,
    };

    let mut stdlib = StdLib::new();

    let small_integer = DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
        unsigned: false,
        zerofill: false,
        size: 16,
    }));

    stdlib.reg_binary_operation(StdLibBinaryOperation {
        operator: BinaryOperator::Plus,
        left: small_integer.clone(),
        right: small_integer.clone(),
        output: small_integer.clone(),
    });

    stdlib.reg_function(
        StdLibFunction::new("max".to_string())
            .gets(vec![small_integer.clone()])
            .returns(small_integer.clone())
            .aggregate()
    );

    stdlib.reg_function(
        StdLibFunction::new("abs".to_string())
            .gets(vec![small_integer.clone()])
            .returns(small_integer.clone())
    );

    let mut source = HashMapSource::new();

    source.simple_insert(
        Path::empty(),
        "index.n",
        "\
            pub struct Complex(double, double)

            fn alpha() {
                let a: small integer := abs(2 + -3);
            }
        ",
    );

    let project = ProjectContext::new(SyncRef::new(stdlib));
    project.request_resolving_module(Path::empty());
    let result = project.resolve(&mut source);
    match result {
        Ok(_) => println!("Project resolved!"),
        Err(errors) => {
            println!("Got errors:");
            for error in errors {
                println!("{}", error);
            }
        }
    }
}
