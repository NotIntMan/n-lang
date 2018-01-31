#![feature(rustc_private)]
#![feature(conservative_impl_trait)]
#![feature(plugin)]
#![cfg_attr(test, plugin(stainless))]

include!("./lib.rs");

fn main() {
    println!("Hello, world!");
}
