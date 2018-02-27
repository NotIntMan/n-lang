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

#[cfg(test)]
mod tests {
    use helpers::assertion::Assertion;
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
    use man_lang::data_sources::DataSource;
    use man_lang::other_requests::{
        deleting,
        inserting,
        InsertingPriority,
        InsertingSource,
        updating,
    };
    use man_lang::selections::{
        SelectionResult,
        SelectionResultSize,
    };

    fn assert_table(source: &DataSource, table_name: &str, table_alias: Option<&str>) {
        match source {
            &DataSource::Table { name, alias } => {
                assert_eq!(name, table_name);
                assert_eq!(alias, table_alias);
            },
            ref o => panic!("This is not table {:?}", o),
        }
    }

    #[test]
    fn simple_updating_query_parses_correctly() {
        let tokens = Scanner::scan("update foo set a.x = 2")
            .expect("Scanner result must be ok");
        let update = parse(tokens.as_slice(), updating)
            .expect("Parser result must be ok");
        assert_eq!(update.low_priority, false);
        assert_eq!(update.ignore, false);
        assert_table(&update.source, "foo", None);
        update.assignments.assert(&[
            ("a.x", Some("2"))
        ]);
        assert_eq!(update.where_clause, None);
        assert_eq!(update.order_by_clause, None);
        assert_eq!(update.limit_clause, None);
    }

    #[test]
    fn simple_inserting_values_query_parses_correctly() {
        let tokens = Scanner::scan("insert into foo(start.x, end.z) values (1, 2), (2, 3), (3, 4)")
            .expect("Scanner result must be ok");
        let insert = parse(tokens.as_slice(), inserting)
            .expect("Parser result must be ok");
        assert_eq!(insert.priority, InsertingPriority::Usual);
        assert_eq!(insert.ignore, false);
        assert_table(&insert.target, "foo", None);
        match &insert.source {
            &InsertingSource::ValueLists { ref properties, ref lists } => {
                assert_eq!(*properties, Some(vec![
                    vec!["start", "x"],
                    vec!["end", "z"],
                ]));
                let mut list_iterator = lists.iter();
                let list = list_iterator.next()
                    .expect("List of lists must contain list");
                list.as_slice().assert(&["1", "2"]);
                let list = list_iterator.next()
                    .expect("List of lists must contain list");
                list.as_slice().assert(&["2", "3"]);
                let list = list_iterator.next()
                    .expect("List of lists must contain list");
                list.as_slice().assert(&["3", "4"]);
                assert_eq!(list_iterator.next(), None);
            },
            o => panic!("Pattern InsertingSource::ValueLists not matches value {:?}", o),
        }
        assert_eq!(insert.on_duplicate_key_update, None);
    }

    #[test]
    fn simple_inserting_assigned_values_query_parses_correctly() {
        let tokens = Scanner::scan("insert high_priority into foo set start.x = 1, end.z = 2 on duplicate key update start.x = 1, end.z = 3")
            .expect("Scanner result must be ok");
        let insert = parse(tokens.as_slice(), inserting)
            .expect("Parser result must be ok");
        assert_eq!(insert.priority, InsertingPriority::High);
        assert_eq!(insert.ignore, false);
        assert_table(&insert.target, "foo", None);
        match &insert.source {
            &InsertingSource::AssignmentList { ref assignments } => {
                assignments.as_slice().assert(&[
                    ("start.x", Some("1")),
                    ("end.z", Some("2")),
                ]);
            },
            o => panic!("Pattern InsertingSource::AssignmentList not matches value {:?}", o),
        }
        insert.on_duplicate_key_update
            .expect("On-duplicate-key-update clause must contain assignments")
            .as_slice()
            .assert(&[
                ("start.x", Some("1")),
                ("end.z", Some("3")),
            ]);
    }

    #[test]
    fn simple_inserting_from_selection_query_parses_correctly() {
        let tokens = Scanner::scan("insert delayed ignore into foo(start.x, end.z) select * from bar")
            .expect("Scanner result must be ok");
        let insert = parse(tokens.as_slice(), inserting)
            .expect("Parser result must be ok");
        assert_eq!(insert.priority, InsertingPriority::Delayed);
        assert_eq!(insert.ignore, true);
        assert_table(&insert.target, "foo", None);
        match &insert.source {
            &InsertingSource::Selection { ref properties, ref query } => {
                assert_eq!(*properties, Some(vec![
                    vec!["start", "x"],
                    vec!["end", "z"],
                ]));
                assert_eq!(query.distinct, false);
                assert_eq!(query.high_priority, false);
                assert_eq!(query.straight_join, false);
                assert_eq!(query.result_size, SelectionResultSize::Usual);
                assert_eq!(query.cache, false);
                assert_eq!(query.result, SelectionResult::All);
                assert_table(&query.source, "bar", None);
                assert_eq!(query.where_clause, None);
                assert_eq!(query.group_by_clause, None);
                assert_eq!(query.having_clause, None);
                assert_eq!(query.order_by_clause, None);
                assert_eq!(query.limit_clause, None);
            },
            o => panic!("Pattern InsertingSource::Selection not matches value {:?}", o),
        }
        assert_eq!(insert.on_duplicate_key_update, None);
    }

    #[test]
    fn simple_deleting_query_parses_correctly() {
        let tokens = Scanner::scan("delete quick from bar where 42 > 80")
            .expect("Scanner result must be ok");
        let delete = parse(tokens.as_slice(), deleting)
            .expect("Parser result must be ok");
        assert_eq!(delete.low_priority, false);
        assert_eq!(delete.quick, true);
        assert_eq!(delete.ignore, false);
        assert_table(&delete.source, "bar", None);
        delete.where_clause
            .clone().expect("Where clause must contain an expression")
            .assert("42 > 80");
        assert_eq!(delete.order_by_clause, None);
        assert_eq!(delete.limit_clause, None);
    }
}
