extern crate indexmap;
extern crate n_lang;

use indexmap::IndexMap;
use n_lang::{
    code_generation::{
        DatabaseProject,
        RPCProject,
    },
    helpers::{
        PathBuf,
        Resolve,
        SyncRef,
    },
    language::{
        BinaryOperator,
        DataType,
        NumberType,
        PrimitiveDataType,
    },
    project_analysis::{
        HashMapSource,
        Module,
        ProjectContext,
        SemanticError,
        StdLib,
        StdLibBinaryOperation,
        StdLibFunction,
    },
};
use std::path::Path;

fn get_test_stdlib() -> StdLib {
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

    stdlib
}

fn get_sources(directory: &str) -> HashMapSource {
    let dir_path = Path::new("./tests/").join(Path::new(directory));
    match HashMapSource::for_dir(&dir_path) {
        Ok(result) => result,
        Err(_) => panic!("Cannot process directory {:#}", directory),
    }
}

fn resolve_directory(stdlib: StdLib, directory: &str) -> Result<IndexMap<SyncRef<PathBuf>, SyncRef<Module>>, Vec<SemanticError>> {
    let source = get_sources(directory);
    let project = ProjectContext::new(SyncRef::new(stdlib));
    for (module_path, _) in source.texts() {
        project.request_resolving_module(module_path.as_path());
    }
    project.resolve(&source)
}

#[test]
fn dir_resolve() {
    let project = match resolve_directory(get_test_stdlib(), "dir_resolve") {
        Ok(project) => project,
        Err(errors) => {
            println!("Got errors:");
            for error in errors {
                println!("{}", error);
            }
            panic!("Resolved some errors");
        }
    };
    let db = DatabaseProject::new(&project);
    let _rpc = RPCProject::new(&project);

    let db_code = db.generate_string()
        .expect("Cannot generate output for database");

    println!("{}", db_code);
}
