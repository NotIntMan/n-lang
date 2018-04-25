#[test]
fn do_it() {
    use helpers::resolve::Resolve;
    use helpers::path::Path;
    use project_analysis::{
        ProjectContext,
        HashMapSource,
    };

    let mut source = HashMapSource::new();

    source.simple_insert(
        Path::new("", "::"),
        "index.n",
        "\
            use complex::*;

            struct Wave {
                freq: integer,
                signal: Complex,
            }
        ",
    );

    source.simple_insert(
        Path::new("complex", "::"),
        "complex.n",
        "\
            pub struct Complex(double, double)
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
