#![feature(range_contains)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;
extern crate env_logger;
#[allow(unused_imports)]
#[macro_use]
pub extern crate nom;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[macro_use]
pub mod helpers;
pub mod lexeme_scanner;
#[macro_use]
pub mod parser_basics;
pub mod desc_lang;
pub mod man_lang;
