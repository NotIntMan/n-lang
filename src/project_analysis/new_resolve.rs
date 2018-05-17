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

    let boolean = DataType::Primitive(PrimitiveDataType::Number(NumberType::Boolean));

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
    );

    let mut source = HashMapSource::new();

    source.simple_insert(
        Path::empty(),
        "index.n",
        "\
            pub struct Complex(small integer, small integer)

            table Waves {
                #[primary_key]
                #[auto_increment]
                id: unsigned integer,
                signal: Complex,
            }

            fn alpha(x: boolean): integer {
                let a := SELECT
                        w.id,
                        max(w.signal.component1) as max_c1
                    FROM Waves w
                    GROUP BY w.id
                    HAVING max(w.signal.component0) > 0
                ;
                let b := SELECT sum(a.max_c1) FROM a;
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
