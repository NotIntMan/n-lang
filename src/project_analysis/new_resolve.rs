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
    );

    let mut source = HashMapSource::new();

    source.simple_insert(
        Path::empty(),
        "index.n",
        "\
            pub struct PersonInfo {
                age: unsigned tiny integer,
            }

            table Users {
                #[primary_key]
                #[auto_increment]
                id: unsigned integer,
                person_info: PersonInfo,
            }

            fn user_age(user: Users::entity): unsigned tiny integer {
                return user.person_info.age;
            }

            fn get_max_user_age(): small integer {
                let t := select max(user_age(u)) from Users u;
                return t.component0;
            }

            fn add_user(person_info: PersonInfo) {
                insert into Users u (u.person_info) values (person_info);
            }

            fn new_person_info(age: unsigned tiny integer): PersonInfo {
                let result: PersonInfo;
                result.age := age;
                return result;
            }

            fn old_all_users(increment: unsigned tiny integer) {
                update Users u set u.person_info.age = user_age(u) + 1;
            }

            fn wrong() {
                let a := select old_all_users(2) from Users;
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
