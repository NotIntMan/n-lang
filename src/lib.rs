#![feature(range_contains)]
#![feature(ptr_internals)]
#![feature(box_syntax)]

extern crate anymap;
extern crate indexmap;
#[cfg(feature = "parser_trace")]
#[macro_use]
extern crate log;
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
pub mod language;
pub mod project_analysis;
pub mod code_generation;
