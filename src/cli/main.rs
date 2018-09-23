extern crate clap;
extern crate n_lang;
extern crate env_logger;

mod cli;
mod stdlib;
mod resolve;

use std::{
    error::Error,
    fs::File,
    io::Write,
    path::Path,
    process::exit,
};

fn write(filename: &Path, content: &str) -> Result<(), Box<Error>> {
    let mut file = File::create(filename)?;
    file.write(content.as_bytes())?;
    Ok(())
}

fn do_it() -> Result<(), Box<Error>> {
    let config = cli::match_cli_config();
    let (db, rpc) = resolve::resolve_dir(&config.projects_dir)?;
    write(&config.output_tsql_file, &db.generate_string()?)?;
    write(&config.output_typescript_file, &rpc.generate_string()?)?;
    Ok(())
}

fn main() {
    env_logger::init();
    match do_it() {
        Ok(_) => println!("Success!"),
        Err(error) => {
            println!("Error: {}", error);
            exit(1);
        }
    }
}
