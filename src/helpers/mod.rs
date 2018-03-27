//! Модуль, содержащий в себе набор простых структур, функций и макросов-помощников, используемых несколькими модулями.

#[macro_use]
pub mod array_macro;
pub mod assertion;
#[macro_use]
pub mod convert_macro;
#[macro_use]
pub mod count_expression_macro;
pub mod extract;
pub mod group;
pub mod display_list;
#[macro_use]
pub mod match_it_macro;
pub mod into_static;
pub mod find_index;