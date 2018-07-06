//! Модуль, содержащий в себе набор простых структур, функций и макросов-помощников, используемых несколькими модулями.

pub use self::{
    as_unique::*,
    assertion::*,
    code_formatter::*,
    display_list::*,
    extract::*,
    extractor::*,
    find_index::*,
    group::*,
    id_pull::*,
    into_static::*,
    is_f32_enough::*,
    lazy::*,
    loud_rw_lock::*,
    map::*,
    parse_component::*,
    parse_number_literal::*,
    path::*,
    re_entrant_rw_lock::*,
    resolve::*,
    result_collect::*,
    sync_ref::*,
    write_pad::*,
};

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
pub mod loud_rw_lock;
pub mod write_pad;
pub mod extractor;
pub mod re_entrant_rw_lock;
pub mod lazy;
pub mod id_pull;
pub mod sync_ref;
pub mod resolve;
pub mod as_unique;
pub mod path;
pub mod parse_component;
pub mod parse_number_literal;
pub mod is_f32_enough;
pub mod result_collect;
pub mod code_formatter;
pub mod map;
#[macro_use]
pub mod universal_assert_macro;

