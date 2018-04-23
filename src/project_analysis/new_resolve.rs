#[test]
fn do_it() {
    use helpers::resolve::Resolve;
    use helpers::sync_ref::SyncRef;
    use project_analysis::{
        Project,
        Text,
        UnresolvedModule,
    };

    let mut project = SyncRef::new(Project {});
    let result = UnresolvedModule::from_text(Text::new("index.n", "\
        struct Complex(double, double)

        struct Wave {
            freq: integer,
            signal: Complex,
        }
    "))
        .unwrap()
        .resolve(&mut project)
    ;
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
