#[macro_use]
extern crate nom;
#[cfg(feature="parser_trace")]
#[macro_use]
extern crate log;
#[macro_use]
extern crate n_lang;

pub mod prettify;
pub mod cool_stuff;

use std::process::exit;
use cool_stuff::{read_line, do_parse};

fn main() {
    println!("Вводите одно за другим, каждое в новой строке.");
    let exit_code = loop {
        let input = match read_line() {
            Ok(input) => input,
            Err(code) => break code,
        };
        match do_parse(&input) {
            Ok(pretty) => println!("{}", pretty),
            Err(error) => println!("{}", error),
        }
    };
    exit(exit_code);
}
