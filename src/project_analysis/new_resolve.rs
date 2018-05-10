#[test]
fn do_it() {
    use helpers::Resolve;
    use helpers::Path;
    use project_analysis::{
        ProjectContext,
        HashMapSource,
    };

    let mut source = HashMapSource::new();

    source.simple_insert(
        Path::new("", "::"),
        "index.n",
        "\
            pub struct Complex(double, double)

            fn alpha() {
                let a := 2;
            }
        ",
    );

    let project = ProjectContext::new();
    project.request_resolving_module(Path::new("", "::"));
    let result = project.resolve(&mut source);
    match result {
        Ok(result) => println!("Resolved: {:#?}", result),
        Err(errors) => {
            println!("Got errors:");
            for error in errors {
                println!("{}", error);
            }
        }
    }
}
