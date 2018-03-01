use lexeme_scanner::Token;
use parser_basics::{
    comma_list,
    keyword,
    none,
    ParserResult,
    symbols,
    u32_literal,
};
use man_lang::data_sources::data_source;
use man_lang::expressions::{
    Expression,
    expression,
};
use man_lang::others::property_path;
use man_lang::selections::{
    selection,
    select_condition,
    select_sorting,
};
use super::*;

parser_rule!(updating_value(i) -> UpdatingValue<'source> {
    alt!(i,
        apply!(keyword, "default") => { |_| UpdatingValue::Default }
        | expression => { |x| UpdatingValue::Expression(x) }
    )
});

parser_rule!(updating_assignment(i) -> UpdatingAssignment<'source> {
    do_parse!(i,
        property: property_path >>
        apply!(symbols, "=") >>
        value: updating_value >>
        (UpdatingAssignment { property, value })
    )
});

parser_rule!(limit_clause(i) -> u32 {
    do_parse!(i,
        apply!(keyword, "limit") >>
        x: u32_literal >>
        (x)
    )
});

/// Выполняет разбор запроса обновления
pub fn updating<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Updating<'source>> {
    do_parse!(input,
        apply!(keyword, "update") >>
        low_priority: opt!(apply!(keyword, "low_priority")) >>
        ignore: opt!(apply!(keyword, "ignore")) >>
        source: data_source >>
        apply!(keyword, "set") >>
        assignments: apply!(comma_list, updating_assignment) >>
        where_clause: opt!(apply!(select_condition, "where")) >>
        order_by_clause: opt!(apply!(select_sorting, "order")) >>
        limit_clause: opt!(limit_clause) >>
        (Updating {
            low_priority: low_priority.is_some(),
            ignore: ignore.is_some(),
            source,
            assignments,
            where_clause,
            order_by_clause,
            limit_clause,
        })
    )
}

parser_rule!(inserting_priority(i) -> InsertingPriority {
    alt!(i,
        apply!(keyword, "low_priority") => { |_| InsertingPriority::Low }
        | apply!(keyword, "delayed") => { |_| InsertingPriority::Delayed }
        | apply!(keyword, "high_priority") => { |_| InsertingPriority::High }
        | none  => { |_| InsertingPriority::Usual }
    )
});

parser_rule!(value_list(i) -> Vec<Expression<'source>> {
    do_parse!(i,
        apply!(symbols, "(") >>
        result: apply!(comma_list, expression) >>
        apply!(symbols, ")") >>
        (result)
    )
});

parser_rule!(property_list(i) -> Vec<Vec<&'source str>> {
    do_parse!(i,
        apply!(symbols, "(") >>
        result: apply!(comma_list, property_path) >>
        apply!(symbols, ")") >>
        (result)
    )
});

parser_rule!(inserting_source(i) -> InsertingSource<'source> {
    alt!(i,
        do_parse!(
            properties: opt!(property_list) >>
            alt!(apply!(keyword, "value") | apply!(keyword, "values")) >>
            lists: apply!(comma_list, value_list) >>
            (InsertingSource::ValueLists { properties, lists })
        )
        | do_parse!(
            apply!(keyword, "set") >>
            assignments: apply!(comma_list, updating_assignment) >>
            (InsertingSource::AssignmentList { assignments })
        )
        | do_parse!(
            properties: opt!(property_list) >>
            query: selection >>
            (InsertingSource::Selection { properties, query })
        )
    )
});

parser_rule!(inserting_on_duplicate_key_update(i) -> Vec<UpdatingAssignment<'source>> {
    do_parse!(i,
        apply!(keyword, "on") >>
        apply!(keyword, "duplicate") >>
        apply!(keyword, "key") >>
        apply!(keyword, "update") >>
        assignments: apply!(comma_list, updating_assignment) >>
        (assignments)
    )
});

/// Выполняет разбор запроса записи
pub fn inserting<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Inserting<'source>> {
    do_parse!(input,
        apply!(keyword, "insert") >>
        priority: inserting_priority >>
        ignore: opt!(apply!(keyword, "ignore")) >>
        apply!(keyword, "into") >>
        target: data_source >>
        source: inserting_source >>
        on_duplicate_key_update: opt!(inserting_on_duplicate_key_update) >>
        (Inserting {
            priority,
            ignore: ignore.is_some(),
            target,
            source,
            on_duplicate_key_update,
        })
    )
}

/// Выполняет разбор запроса удаления
pub fn deleting<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Deleting<'source>> {
    do_parse!(input,
        apply!(keyword, "delete") >>
        low_priority: opt!(apply!(keyword, "low_priority")) >>
        quick: opt!(apply!(keyword, "quick")) >>
        ignore: opt!(apply!(keyword, "ignore")) >>
        apply!(keyword, "from") >>
        source: data_source >>
        where_clause: opt!(apply!(select_condition, "where")) >>
        order_by_clause: opt!(apply!(select_sorting, "order")) >>
        limit_clause: opt!(limit_clause) >>
        (Deleting {
            low_priority: low_priority.is_some(),
            quick: quick.is_some(),
            ignore: ignore.is_some(),
            source,
            where_clause,
            order_by_clause,
            limit_clause,
        })
    )
}
