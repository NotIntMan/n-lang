#![feature(range_contains)]
#![feature(rustc_private)]
#![feature(conservative_impl_trait)]
#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]
#![feature(const_fn)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;
extern crate env_logger;
#[allow(unused_imports)]
#[macro_use]
pub extern crate nom;

#[macro_use]
pub mod helpers;
pub mod lexeme_scanner;
#[macro_use]
pub mod parser_basics;
pub mod desc_lang;
