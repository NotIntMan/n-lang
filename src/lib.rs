#![feature(range_contains)]

#[cfg(feature="parser_trace")]
#[macro_use]
extern crate log;
#[allow(unused_imports)]
#[macro_use]
pub extern crate nom;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
extern crate indexmap;

#[macro_use]
pub mod helpers;
pub mod lexeme_scanner;
#[macro_use]
pub mod parser_basics;
pub mod desc_lang;
pub mod man_lang;
