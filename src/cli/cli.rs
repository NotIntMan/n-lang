pub mod validators {
    use std::path::Path;

    pub fn is_dir(value: String) -> Result<(), String> {
        let path = Path::new(&value);
        if path.is_dir() {
            Ok(())
        } else {
            Err("Value should be an existing directory".into())
        }
    }
}

use clap::{Arg, App, ArgMatches};
use std::path::PathBuf;

pub const PROJECT_DIR: &'static str = "Project's dir";
pub const OUTPUT_TS_FILE: &'static str = "Output TypeScript file";
pub const OUTPUT_SQL_FILE: &'static str = "Output T-SQL file";

pub fn build_cli_app<'a, 'b>() -> App<'a, 'b> {
    App::new("N-lang compiler")
        .version("0.1.0")
        .author("Dmitry Demin <shepardiwe@gmail.com>")
        .about("Compiles N-lang projects.")
        .arg(
            Arg::with_name(PROJECT_DIR)
                .help("Project's directory location")
                .required(true)
                .index(1)
                .validator(validators::is_dir)
        )
        .arg(
            Arg::with_name(OUTPUT_TS_FILE)
                .help("Destination location for result TypeScript file")
                .required(true)
                .index(2)
        )
        .arg(
            Arg::with_name(OUTPUT_SQL_FILE)
                .help("Destination location for result T-SQL file")
                .required(true)
                .index(3)
        )
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CLIConfig {
    pub projects_dir: PathBuf,
    pub output_typescript_file: PathBuf,
    pub output_tsql_file: PathBuf,
}

fn extract_required_param<'a>(matches: &'a ArgMatches, param: &str) -> &'a str {
    match matches.value_of(param) {
        Some(res) => res,
        None => panic!("<{}> is required", param),
    }
}

pub fn match_cli_config() -> CLIConfig {
    let matches = build_cli_app().get_matches();
    CLIConfig {
        projects_dir: PathBuf::from(extract_required_param(&matches, PROJECT_DIR)),
        output_typescript_file: PathBuf::from(extract_required_param(&matches, OUTPUT_TS_FILE)),
        output_tsql_file: PathBuf::from(extract_required_param(&matches, OUTPUT_SQL_FILE)),
    }
}
