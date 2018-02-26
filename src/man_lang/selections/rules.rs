use lexeme_scanner::Token;
use parser_basics::{
    comma_list,
    identifier,
    keyword,
    none,
    ParserResult,
    symbols,
    u32_literal,
};
use man_lang::expressions::{expression, Expression};
use man_lang::data_sources::data_source;
use super::*;

parser_rule!(select_distincty(i) -> bool {
    alt!(i,
        apply!(keyword, "all") => { |_| false } |
        apply!(keyword, "distinct") => { |_| true } |
        apply!(keyword, "distinctrow") => { |_| true } |
        none => { |_| false }
    )
});

parser_rule!(select_result_size(i) -> SelectionResultSize {
    alt!(i,
        apply!(keyword, "sql_small_result") => { |_| SelectionResultSize::Small } |
        apply!(keyword, "sql_big_result") => { |_| SelectionResultSize::Big } |
        apply!(keyword, "sql_buffer_result") => { |_| SelectionResultSize::Buffered } |
        none => { |_| SelectionResultSize::Usual }
    )
});

parser_rule!(select_cache(i) -> bool {
    alt!(i,
        apply!(keyword, "sql_cache") => { |_| true } |
        apply!(keyword, "sql_no_cache") => { |_| false } |
        none => { |_| false }
    )
});

parser_rule!(select_expression(i) -> SelectionExpression<'source> {
    do_parse!(i,
        expr: expression >>
        alias: opt!(do_parse!(
            apply!(keyword, "as") >>
            name: identifier >>
            (name)
        )) >>
        (SelectionExpression { expr, alias })
    )
});

parser_rule!(select_result(i) -> SelectionResult<'source> {
    alt!(i,
        apply!(symbols, "*") => { |_| SelectionResult::All }
        | apply!(comma_list, select_expression) => { |x| SelectionResult::Some(x) }
    )
});

parser_rule!(select_condition(i, prefix_keyword_text: &str) -> Expression<'source> {
    do_parse!(i,
        apply!(keyword, prefix_keyword_text) >>
        expr: expression >>
        (expr)
    )
});

// TODO Рассмотреть возможность использования синтаксиса tableName.* в группировке и сортировке
parser_rule!(select_sorting_order(i) -> SelectionSortingOrder {
    alt!(i,
        apply!(keyword, "asc") => { |_| SelectionSortingOrder::Asc } |
        apply!(keyword, "desc") => { |_| SelectionSortingOrder::Desc } |
        none => { |_| SelectionSortingOrder::Asc }
    )
});

parser_rule!(select_sorting_item(i) -> SelectionSortingItem<'source> {
    do_parse!(i,
        expr: expression >>
        order: select_sorting_order >>
        (SelectionSortingItem { expr, order })
    )
});

parser_rule!(select_sorting(i, prefix_keyword_text: &str) -> Vec<SelectionSortingItem<'source>> {
    do_parse!(i,
        apply!(keyword, prefix_keyword_text) >>
        apply!(keyword, "by") >>
        items: apply!(comma_list, select_sorting_item) >>
        (items)
    )
});

parser_rule!(select_group_by_clause(i) -> SelectionGroupByClause<'source> {
    do_parse!(i,
        sorting: apply!(select_sorting, "group") >>
        with_rollup: opt!(do_parse!(
            apply!(keyword, "with") >>
            apply!(keyword, "rollup") >>
            (())
        )) >>
        (SelectionGroupByClause { sorting, with_rollup: with_rollup.is_some() })
    )
});

parser_rule!(selection_limit(i) -> SelectionLimit {
    do_parse!(i,
        apply!(keyword, "limit") >>
        a: u32_literal >>
        x: alt!(
            do_parse!(
                apply!(symbols, ",") >>
                count: u32_literal >>
                (SelectionLimit { offset: Some(a), count })
            )
            | do_parse!(
                apply!(keyword, "offset") >>
                offset: u32_literal >>
                (SelectionLimit { offset: Some(offset), count: a })
            )
            | none => { |_| SelectionLimit { offset: None, count: a } }
        ) >>
        (x)
    )
});

/// Функция, выполняющая разбор запроса выборки
pub fn selection<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Selection<'source>> {
    do_parse!(input,
        apply!(keyword, "select") >>
        distinct: select_distincty >>
        high_priority: opt!(apply!(keyword, "high_priority")) >>
        straight_join: opt!(apply!(keyword, "straight_join")) >>
        result_size: select_result_size >>
        cache: select_cache >>
        result: select_result >>
        apply!(keyword, "from") >>
        source: data_source >>
        where_clause: opt!(apply!(select_condition, "where")) >>
        group_by_clause: opt!(select_group_by_clause) >>
        having_clause: opt!(apply!(select_condition, "having")) >>
        order_by_clause: opt!(apply!(select_sorting, "order")) >>
        limit_clause: opt!(selection_limit) >>
        (Selection {
            distinct,
            high_priority: high_priority.is_some(),
            straight_join: straight_join.is_some(),
            result_size,
            cache,
            result,
            source,
            where_clause,
            group_by_clause,
            having_clause,
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
    use man_lang::data_sources::{
        DataSource,
        JoinType,
        JoinCondition,
    };
    use man_lang::selections::{
        selection,
        SelectionGroupByClause,
        SelectionLimit,
        SelectionResult,
        SelectionResultSize,
        SelectionSortingItem,
        SelectionSortingOrder,
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
    fn simple_selection_parses_correctly() {
        let tokens = Scanner::scan(
            "select * from foo"
        ).expect("Scanner result must be ok");
        let query = parse(tokens.as_slice(), selection)
            .expect("Parser result must be ok");

        assert_eq!(query.distinct, false);
        assert_eq!(query.high_priority, false);
        assert_eq!(query.straight_join, false);
        assert_eq!(query.result_size, SelectionResultSize::Usual);
        assert_eq!(query.cache, false);
        assert_eq!(query.result, SelectionResult::All);
        assert_table(&query.source, "foo", None);
        assert_eq!(query.where_clause, None);
        assert_eq!(query.group_by_clause, None);
        assert_eq!(query.having_clause, None);
        assert_eq!(query.order_by_clause, None);
        assert_eq!(query.limit_clause, None);
    }

    #[test]
    fn simple_selection_with_flags_parses_correctly() {
        let tokens = Scanner::scan(
            "select distinct high_priority straight_join sql_big_result sql_cache * from foo"
        ).expect("Scanner result must be ok");
        let query = parse(tokens.as_slice(), selection)
            .expect("Parser result must be ok");

        assert_eq!(query.distinct, true);
        assert_eq!(query.high_priority, true);
        assert_eq!(query.straight_join, true);
        assert_eq!(query.result_size, SelectionResultSize::Big);
        assert_eq!(query.cache, true);
        assert_eq!(query.result, SelectionResult::All);
        assert_table(&query.source, "foo", None);
        assert_eq!(query.where_clause, None);
        assert_eq!(query.group_by_clause, None);
        assert_eq!(query.having_clause, None);
        assert_eq!(query.order_by_clause, None);
        assert_eq!(query.limit_clause, None);
    }

    #[test]
    fn simple_selection_with_filtering_parses_correctly() {
        let tokens = Scanner::scan(
            "select * from foo where id = 2"
        ).expect("Scanner result must be ok");
        let query = parse(tokens.as_slice(), selection)
            .expect("Parser result must be ok");

        assert_eq!(query.distinct, false);
        assert_eq!(query.high_priority, false);
        assert_eq!(query.straight_join, false);
        assert_eq!(query.result_size, SelectionResultSize::Usual);
        assert_eq!(query.cache, false);
        assert_eq!(query.result, SelectionResult::All);
        assert_table(&query.source, "foo", None);
        query.where_clause
            .expect("Where clause should contain an expression")
            .assert("id = 2");
        assert_eq!(query.group_by_clause, None);
        assert_eq!(query.having_clause, None);
        assert_eq!(query.order_by_clause, None);
        assert_eq!(query.limit_clause, None);
    }

    fn assert_selection_sorting_items(items: &Vec<SelectionSortingItem>, pattern: Vec<(&str, SelectionSortingOrder)>) {
        let mut pattern_iter = pattern.iter();
        for item in items {
            let &(expression_text, order) = pattern_iter.next()
                .expect("Pattern should have same length as the items vector");
            item.expr.assert(expression_text);
            assert_eq!(item.order, order);
        }
    }

    fn assert_selection_result(items: &SelectionResult, pattern: Vec<(&str, Option<&str>)>) {
        match items {
            &SelectionResult::Some(ref items) => {
                let mut pattern_iter = pattern.iter();
                for item in items {
                    let &(expression_text, alias) = pattern_iter.next()
                        .expect("Pattern should have same length as the items vector");
                    item.expr.assert(expression_text);
                    assert_eq!(item.alias, alias);
                }
            },
            &SelectionResult::All => panic!("SelectionResult::All is not expected."),
        }
    }

    fn assert_selection_group_by_clause(clause: &SelectionGroupByClause, pattern: Vec<(&str, SelectionSortingOrder)>, with_rollup: bool) {
        assert_eq!(clause.with_rollup, with_rollup);
        assert_selection_sorting_items(&clause.sorting, pattern);
    }

    #[test]
    fn complex_query_parses_correctly() {
        const QUERY: &'static str = "select distinct
              m.name as man_name,
              i.name as item_name,
              s.max_cost as max_cost
            from (
              select
                c.man_id,
                max(c.cost) as max_cost
              from Costs c
              group by
                c.man_id asc
                with rollup
              having max(c.cost) > 5
            ) s
              inner join Costs c on (s.man_id is null or s.man_id = c.man_id) and s.max_cost = c.cost
              left join Mans m on c.man_id = m.man_id
              left join Items i using(item_id)
            where s.max_cost < 100
            order by
              m.name asc,
              i.name desc
            limit 100";

        let tokens = Scanner::scan(QUERY).expect("Scanner result must be ok");
        let query = parse(tokens.as_slice(), selection)
            .expect("Parser result must be ok");

        assert_eq!(query.distinct, true);
        assert_eq!(query.high_priority, false);
        assert_eq!(query.straight_join, false);
        assert_eq!(query.result_size, SelectionResultSize::Usual);
        assert_eq!(query.cache, false);
        assert_selection_result(&query.result, vec![
            ("m.name", Some("man_name")),
            ("i.name", Some("item_name")),
            ("s.max_cost", Some("max_cost")),
        ]);
        query.where_clause
            .expect("Where clause should contain an expression")
            .assert("s.max_cost < 100");
        assert_selection_sorting_items(
            &query.order_by_clause
                .expect("Order-by clause should contain items"),
            vec![
                ("m.name", SelectionSortingOrder::Asc),
                ("i.name", SelectionSortingOrder::Desc),
            ]
        );
        assert_eq!(query.having_clause, None);
        assert_eq!(query.limit_clause, Some(SelectionLimit {
            offset: None,
            count: 100,
        }));
        let subquery_0 = match &query.source {
            &DataSource::Join {
                join_type: JoinType::Left,
                condition: Some(JoinCondition::Using(ref cond)),
                ref left,
                ref right,
            } => {
                assert_eq!(*cond, vec![vec!["item_id"]]);
                assert_table(&**right, "Items", Some("i"));
                (**left).clone()
            },
            o => panic!("Wrong subquery! {:?}", o),
        };
        let subquery_1 = match subquery_0 {
            DataSource::Join {
                join_type: JoinType::Left,
                condition: Some(JoinCondition::Expression(ref cond)),
                ref left,
                ref right,
            } => {
                cond.assert("c.man_id = m.man_id");
                assert_table(&**right, "Mans", Some("m"));
                (**left).clone()
            },
            o => panic!("Wrong subquery! {:?}", o),
        };
        let subquery_2 = match subquery_1 {
            DataSource::Join {
                join_type: JoinType::Cross,
                condition: Some(JoinCondition::Expression(ref cond)),
                ref left,
                ref right,
            } => {
                cond.assert("(s.man_id is null or s.man_id = c.man_id) and s.max_cost = c.cost");
                assert_table(&**right, "Costs", Some("c"));
                (**left).clone()
            },
            o => panic!("Wrong subquery! {:?}", o),
        };
        let subquery_3 = match subquery_2 {
            DataSource::Selection { ref query, ref alias } => {
                assert_eq!(*alias, "s");
                (**query).clone()
            },
            o => panic!("Wrong subquery! {:?}", o),
        };
        assert_eq!(subquery_3.distinct, false);
        assert_eq!(subquery_3.high_priority, false);
        assert_eq!(subquery_3.straight_join, false);
        assert_eq!(subquery_3.result_size, SelectionResultSize::Usual);
        assert_eq!(subquery_3.cache, false);
        assert_table(&subquery_3.source, "Costs", Some("c"));
        assert_eq!(subquery_3.where_clause, None);
        assert_selection_group_by_clause(
            &subquery_3.group_by_clause
                .clone()
                .expect("Group-by clause should contain items"),
            vec![("c.man_id", SelectionSortingOrder::Asc)],
            true
        );
        subquery_3.having_clause
            .clone()
            .expect("Subquery's having clause should contain an expression")
            .assert("max(c.cost) > 5");
        assert_eq!(subquery_3.order_by_clause, None);
        assert_eq!(subquery_3.limit_clause, None);
    }
}
