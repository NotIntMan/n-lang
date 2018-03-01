#[macro_use]
extern crate n_transpiler;
extern crate indexmap;
#[macro_use]
extern crate pretty_assertions;

use n_transpiler::helpers::assertion::Assertion;
use n_transpiler::man_lang::expressions::*;
use n_transpiler::man_lang::data_sources::*;
use n_transpiler::man_lang::selections::*;

fn extract_literal<'source>(expr: Expression<'source>) -> LiteralType {
    match_it!(expr, Expression::Literal(lit) => lit.literal_type)
}

#[test]
fn number_literals_parses_correctly() {
    {
        assert_eq!(
            extract_literal(parse!("-2", expression)),
            LiteralType::NumberLiteral {
                negative: true,
                fractional: false,
                radix: 10,
            }
        );
    }
    {
        assert_eq!(
            extract_literal(parse!("0b101.1", expression)),
            LiteralType::NumberLiteral {
                negative: false,
                fractional: true,
                radix: 2,
            }
        );
    }
}

#[test]
fn string_and_braced_literals_parses_correctly() {
    {
        assert_eq!(
            extract_literal(parse!("\"azaz\"", expression)),
            LiteralType::StringLiteral { length: 4 }
        );
    }
    {
        assert_eq!(
            extract_literal(parse!("'can\nzas'", expression)),
            LiteralType::BracedExpressionLiteral { length: 7 }
        );
    }
}

#[test]
fn keyword_literals_parses_correctly() {
    {
        assert_eq!(
            extract_literal(parse!("true", expression)),
            LiteralType::KeywordLiteral(KeywordLiteralType::True)
        );
    }
    {
        assert_eq!(
            extract_literal(parse!("false", expression)),
            LiteralType::KeywordLiteral(KeywordLiteralType::False)
        );
    }
    {
        assert_eq!(
            extract_literal(parse!("null", expression)),
            LiteralType::KeywordLiteral(KeywordLiteralType::Null)
        );
    }
}

fn assert_identifier<'source>(expr: Expression, text: &str) {
    match_it!(expr, Expression::Identifier(token) => assert_eq!(token.text, text));
}

fn extract_prefix_unary_expression<'source>(expr: Expression<'source>, operator: PrefixUnaryOperator) -> Expression<'source> {
    match_it!(expr, Expression::PrefixUnaryOperation(op, expr) => {
        assert_eq!(op, operator);
        *expr
    })
}

fn extract_postfix_unary_expression<'source>(expr: Expression<'source>, operator: PostfixUnaryOperator) -> Expression<'source> {
    match_it!(expr, Expression::PostfixUnaryOperation(op, expr) => {
        assert_eq!(op, operator);
        *expr
    })
}

#[test]
fn all_unary_operations_parses_correctly() {
    let mut expr = parse!(
        "!all any + - ~ binary row exists \
        data \
        is unknown is false is not unknown is not false is true is not true is not null is null",
        expression
    );
    expr = extract_postfix_unary_expression(expr, PostfixUnaryOperator::IsNull);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Not);
    expr = extract_postfix_unary_expression(expr, PostfixUnaryOperator::IsNull);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Not);
    expr = extract_postfix_unary_expression(expr, PostfixUnaryOperator::IsTrue);
    expr = extract_postfix_unary_expression(expr, PostfixUnaryOperator::IsTrue);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Not);
    expr = extract_postfix_unary_expression(expr, PostfixUnaryOperator::IsFalse);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Not);
    expr = extract_postfix_unary_expression(expr, PostfixUnaryOperator::IsUnknown);
    expr = extract_postfix_unary_expression(expr, PostfixUnaryOperator::IsFalse);
    expr = extract_postfix_unary_expression(expr, PostfixUnaryOperator::IsUnknown);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Not);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::All);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Any);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Plus);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Minus);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Tilde);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Binary);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Row);
    expr = extract_prefix_unary_expression(expr, PrefixUnaryOperator::Exists);
    assert_identifier(expr, "data");
}

fn extract_binary_expression<'source>(expr: Expression<'source>, operator: BinaryOperator) -> (Expression<'source>, Expression<'source>) {
    match_it!(expr, Expression::BinaryOperation(left, op, right) => {
        assert_eq!(op, operator);
        (*left, *right)
    })
}

#[test]
fn simple_infix_expression_parses_correctly() {
    let result = parse!("a and b or c xor d", expression);
    let (left, right) = extract_binary_expression(result, BinaryOperator::Or);
    let (left_left, left_right) = extract_binary_expression(left, BinaryOperator::And);
    assert_identifier(left_left, "a");
    assert_identifier(left_right, "b");
    let (right_left, right_right) = extract_binary_expression(right, BinaryOperator::XOr);
    assert_identifier(right_left, "c");
    assert_identifier(right_right, "d");
}

#[test]
fn complex_infix_expression_parses_correctly() {
    let result = parse!("a and b or c xor d or e ** f", expression);
    let (left, right) = extract_binary_expression(result, BinaryOperator::Or);
    {
        let (left, right) = extract_binary_expression(left, BinaryOperator::Or);
        {
            let (left, right) = extract_binary_expression(left, BinaryOperator::And);
            assert_identifier(left, "a");
            assert_identifier(right, "b");
        }
        {
            let (left, right) = extract_binary_expression(right, BinaryOperator::XOr);
            assert_identifier(left, "c");
            assert_identifier(right, "d");
        }
    }
    {
        let (left, right) = extract_binary_expression(right, BinaryOperator::Pow);
        assert_identifier(left, "e");
        assert_identifier(right, "f");
    }
}

#[test]
fn simple_operations_of_all_types_parses_correctly() {
    fn assert_operation(text: &str, operator: BinaryOperator) {
        let input = format!("left_identifier {} RightIdentifier", text);
        let result = parse!(&input, expression);
        let (left, right) = extract_binary_expression(result, operator);
        assert_identifier(left, "left_identifier");
        assert_identifier(right, "RightIdentifier");
    };
    assert_operation("or", BinaryOperator::Or);
    assert_operation("||", BinaryOperator::Or);
    assert_operation("xor", BinaryOperator::XOr);
    assert_operation("^^", BinaryOperator::XOr);
    assert_operation("and", BinaryOperator::And);
    assert_operation("|", BinaryOperator::BitOr);
    assert_operation("^", BinaryOperator::BitXOr);
    assert_operation("&", BinaryOperator::BitAnd);
    assert_operation("<<", BinaryOperator::ShiftLeft);
    assert_operation(">>", BinaryOperator::ShiftRight);
    assert_operation("is in", BinaryOperator::IsIn);
    assert_operation("=", BinaryOperator::Equals);
    assert_operation(">=", BinaryOperator::MoreThanOrEquals);
    assert_operation(">", BinaryOperator::MoreThan);
    assert_operation("<=", BinaryOperator::LessThanOrEquals);
    assert_operation("<", BinaryOperator::LessThan);
    assert_operation("like", BinaryOperator::Like);
    assert_operation("sounds like", BinaryOperator::SoundsLike);
    assert_operation("regexp", BinaryOperator::RegExp);
    assert_operation("+", BinaryOperator::Plus);
    assert_operation("-", BinaryOperator::Minus);
    assert_operation("*", BinaryOperator::Times);
    assert_operation("/", BinaryOperator::Divide);
    assert_operation("%", BinaryOperator::Mod);
    assert_operation("mod", BinaryOperator::Mod);
    assert_operation("div", BinaryOperator::Div);
    assert_operation("**", BinaryOperator::Pow);
    assert_operation("..", BinaryOperator::Interval);
}

#[test]
fn property_access_and_set_and_function_call_expression_parses_correctly() {
    let result = parse!("foo(bar, bar.baz, (box, boz))", expression);
    let mut args = match_it!(result, Expression::FunctionCall(name, args) => {
        assert_eq!(name, "foo");
        args
    });
    assert_identifier(args.remove(0), "bar");
    let arg1 = match_it!(args.remove(0), Expression::PropertyAccess(expr, prop) => {
        assert_eq!(prop, ["baz"]);
        *expr
    });
    assert_identifier(arg1, "bar");
    let mut arg2 = match_it!(args.remove(0), Expression::Set(vec) => vec);
    assert_identifier(arg2.remove(0), "box");
    assert_identifier(arg2.remove(0), "boz");
}

#[test]
fn simple_join_parses_correctly() {
    let result = parse!("foo f INNER JOIN bar b ON f.foo_id = b.bar_id", data_source);
    let (left, right) = match_it!(result, DataSource::Join { join_type, condition, left, right } => {
        assert_eq!(join_type, JoinType::Cross);
        match_it!(condition, Some(JoinCondition::Expression(condition)) => {
            condition.assert("f.foo_id = b.bar_id");
        });
        (*left, *right)
    });
    match_it!(left, DataSource::Table { name, alias } => {
        assert_eq!(name, "foo");
        assert_eq!(alias, Some("f"));
    });
    match_it!(right, DataSource::Table { name, alias } => {
        assert_eq!(name, "bar");
        assert_eq!(alias, Some("b"));
    });
}

fn assert_table(source: &DataSource, table_name: &str, table_alias: Option<&str>) {
    match_it!(source, &DataSource::Table { name, alias } => {
            assert_eq!(name, table_name);
            assert_eq!(alias, table_alias);
        });
}

#[test]
fn simple_selection_parses_correctly() {
    let query = parse!("select * from foo", selection);

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
    let query = parse!("select distinct high_priority straight_join sql_big_result sql_cache * from foo", selection);

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
    let query = parse!("select * from foo where id = 2", selection);

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
    let mut pattern_iterator = pattern.iter();
    for item in items {
        let &(expression_text, order) = pattern_iterator.next()
            .expect("Pattern should have same length as the items vector");
        item.expr.assert(expression_text);
        assert_eq!(item.order, order);
    }
}

fn assert_selection_result(items: &SelectionResult, pattern: Vec<(&str, Option<&str>)>) {
    match_it!(items, &SelectionResult::Some(ref items) => {
            let mut pattern_iter = pattern.iter();
            for item in items {
                let &(expression_text, alias) = pattern_iter.next()
                    .expect("Pattern should have same length as the items vector");
                item.expr.assert(expression_text);
                assert_eq!(item.alias, alias);
            }
        });
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

    let query = parse!(QUERY, selection);

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
        ],
    );
    assert_eq!(query.having_clause, None);
    assert_eq!(query.limit_clause, Some(SelectionLimit {
        offset: None,
        count: 100,
    }));
    let subquery_0 = match_it!(&query.source, &DataSource::Join {
            join_type: JoinType::Left,
            condition: Some(JoinCondition::Using(ref cond)),
            ref left,
            ref right,
        } => {
            assert_eq!(*cond, vec![vec!["item_id"]]);
            assert_table(&**right, "Items", Some("i"));
            (**left).clone()
        });
    let subquery_1 = match_it!(subquery_0, DataSource::Join {
            join_type: JoinType::Left,
            condition: Some(JoinCondition::Expression(ref cond)),
            ref left,
            ref right,
        } => {
            cond.assert("c.man_id = m.man_id");
            assert_table(&**right, "Mans", Some("m"));
            (**left).clone()
        });
    let subquery_2 = match_it!(subquery_1, DataSource::Join {
            join_type: JoinType::Cross,
            condition: Some(JoinCondition::Expression(ref cond)),
            ref left,
            ref right,
        } => {
            cond.assert("(s.man_id is null or s.man_id = c.man_id) and s.max_cost = c.cost");
            assert_table(&**right, "Costs", Some("c"));
            (**left).clone()
        });
    let subquery_3 = match_it!(subquery_2, DataSource::Selection { ref query, ref alias } => {
            assert_eq!(*alias, "s");
            (**query).clone()
        });
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
        true,
    );
    subquery_3.having_clause
        .clone()
        .expect("Subquery's having clause should contain an expression")
        .assert("max(c.cost) > 5");
    assert_eq!(subquery_3.order_by_clause, None);
    assert_eq!(subquery_3.limit_clause, None);
}
