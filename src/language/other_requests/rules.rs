use lexeme_scanner::Token;
use parser_basics::{
    comma_list,
    item_position,
    keyword,
    none,
    ParserResult,
    symbols,
    symbol_position,
    u32_literal,
};
use language::{
    data_source,
    expression,
    ItemPath,
    selection,
    select_condition,
    select_sorting,
    property_path,
};
use super::*;

parser_rule!(updating_value(i) -> UpdatingValueAST<'source> {
    alt!(i,
        apply!(keyword, "default") => { |t: &Token| UpdatingValueAST::Default(t.pos()) }
        | expression => { |x| UpdatingValueAST::Expression(x) }
    )
});

parser_rule!(updating_assignment(i) -> UpdatingAssignmentAST<'source> {
    do_parse!(i,
        begin: symbol_position >>
        property: property_path >>
        apply!(symbols, "=") >>
        value: updating_value >>
        pos: apply!(item_position, begin) >>
        (UpdatingAssignmentAST { property, value, pos })
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
pub fn updating<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, UpdatingAST<'source>> {
    do_parse!(input,
        begin: symbol_position >>
        apply!(keyword, "update") >>
        low_priority: opt!(apply!(keyword, "low_priority")) >>
        ignore: opt!(apply!(keyword, "ignore")) >>
        source: data_source >>
        apply!(keyword, "set") >>
        assignments: apply!(comma_list, updating_assignment) >>
        where_clause: opt!(apply!(select_condition, "where")) >>
        order_by_clause: opt!(apply!(select_sorting, "order")) >>
        limit_clause: opt!(limit_clause) >>
        pos: apply!(item_position, begin) >>
        (UpdatingAST {
            low_priority: low_priority.is_some(),
            ignore: ignore.is_some(),
            source,
            assignments,
            where_clause,
            order_by_clause,
            limit_clause,
            pos,
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

parser_rule!(value_list(i) -> ValueList<'source> {
    do_parse!(i,
        begin: symbol_position >>
        apply!(symbols, "(") >>
        values: apply!(comma_list, expression) >>
        apply!(symbols, ")") >>
        pos: apply!(item_position, begin) >>
        (ValueList { values, pos })
    )
});

parser_rule!(property_list(i) -> Vec<ItemPath> {
    do_parse!(i,
        apply!(symbols, "(") >>
        result: apply!(comma_list, property_path) >>
        apply!(symbols, ")") >>
        (result)
    )
});

parser_rule!(inserting_source(i) -> InsertingSourceAST<'source> {
    do_parse!(i,
        begin: symbol_position >>
        body: alt!(
            do_parse!(
                properties: opt!(property_list) >>
                alt!(apply!(keyword, "value") | apply!(keyword, "values")) >>
                lists: apply!(comma_list, value_list) >>
                (InsertingSourceASTBody::ValueLists { properties, lists })
            )
            | do_parse!(
                apply!(keyword, "set") >>
                assignments: apply!(comma_list, updating_assignment) >>
                (InsertingSourceASTBody::AssignmentList { assignments })
            )
            | do_parse!(
                properties: opt!(property_list) >>
                query: selection >>
                (InsertingSourceASTBody::Selection { properties, query })
            )
        ) >>
        pos: apply!(item_position, begin) >>
        (InsertingSourceAST { body, pos })
    )
});

parser_rule!(inserting_on_duplicate_key_update(i) -> Vec<UpdatingAssignmentAST<'source>> {
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
pub fn inserting<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, InsertingAST<'source>> {
    do_parse!(input,
        apply!(keyword, "insert") >>
        priority: inserting_priority >>
        ignore: opt!(apply!(keyword, "ignore")) >>
        apply!(keyword, "into") >>
        target: data_source >>
        source: inserting_source >>
        on_duplicate_key_update: opt!(inserting_on_duplicate_key_update) >>
        (InsertingAST {
            priority,
            ignore: ignore.is_some(),
            target,
            source,
            on_duplicate_key_update,
        })
    )
}

/// Выполняет разбор запроса удаления
pub fn deleting<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, DeletingAST<'source>> {
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
        (DeletingAST {
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
