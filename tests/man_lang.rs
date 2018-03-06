#[macro_use]
extern crate n_lang;
extern crate indexmap;
#[macro_use]
extern crate pretty_assertions;

use n_lang::helpers::assertion::Assertion;
use n_lang::syntax_parser::expressions::*;
use n_lang::syntax_parser::data_sources::*;
use n_lang::syntax_parser::selections::*;
use n_lang::syntax_parser::other_requests::*;
use n_lang::syntax_parser::statements::*;

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
        assert_eq!(name, vec!["foo"]);
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
        assert_eq!(name, vec!["foo"]);
        assert_eq!(alias, Some("f"));
    });
    match_it!(right, DataSource::Table { name, alias } => {
        assert_eq!(name, vec!["bar"]);
        assert_eq!(alias, Some("b"));
    });
}

fn assert_table(source: &DataSource, table_name: &str, table_alias: Option<&str>) {
    match_it!(source, &DataSource::Table { ref name, alias } => {
            assert_eq!(*name, vec![table_name]);
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

#[test]
fn simple_updating_query_parses_correctly() {
    let update = parse!("update foo set a.x = 2", updating);
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
    let insert = parse!("insert into foo(start.x, end.z) values (1, 2), (2, 3), (3, 4)", inserting);
    assert_eq!(insert.priority, InsertingPriority::Usual);
    assert_eq!(insert.ignore, false);
    assert_table(&insert.target, "foo", None);
    match_it!(&insert.source, &InsertingSource::ValueLists { ref properties, ref lists } => {
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
    });
    assert_eq!(insert.on_duplicate_key_update, None);
}

#[test]
fn simple_inserting_assigned_values_query_parses_correctly() {
    let insert = parse!("insert high_priority into foo set start.x = 1, end.z = 2 on duplicate key update start.x = 1, end.z = 3", inserting);
    assert_eq!(insert.priority, InsertingPriority::High);
    assert_eq!(insert.ignore, false);
    assert_table(&insert.target, "foo", None);
    match_it!(&insert.source, &InsertingSource::AssignmentList { ref assignments } => {
        assignments.as_slice().assert(&[
            ("start.x", Some("1")),
            ("end.z", Some("2")),
        ]);
    });
    match_it!(&insert.on_duplicate_key_update, &Some(ref assignments) => {
        assignments.as_slice()
            .assert(&[
                ("start.x", Some("1")),
                ("end.z", Some("3")),
            ]);
    });
}

#[test]
fn simple_inserting_from_selection_query_parses_correctly() {
    let insert = parse!("insert delayed ignore into foo(start.x, end.z) select * from bar", inserting);
    assert_eq!(insert.priority, InsertingPriority::Delayed);
    assert_eq!(insert.ignore, true);
    assert_table(&insert.target, "foo", None);
    match_it!(&insert.source, &InsertingSource::Selection { ref properties, ref query } => {
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
    });
    assert_eq!(insert.on_duplicate_key_update, None);
}

#[test]
fn simple_deleting_query_parses_correctly() {
    let delete = parse!("delete quick from bar where 42 > 80", deleting);
    assert_eq!(delete.low_priority, false);
    assert_eq!(delete.quick, true);
    assert_eq!(delete.ignore, false);
    assert_table(&delete.source, "bar", None);
    match_it!(&delete.where_clause, &Some(ref clause) => { clause.assert("42 > 80"); });
    assert_eq!(delete.order_by_clause, None);
    assert_eq!(delete.limit_clause, None);
}

#[test]
fn simple_definition_parses_correctly() {
    let result = parse!("let my_first_variable: boolean := false", statement);
    match_it!(result, Statement::VariableDefinition { name, ref data_type, ref default_value } => {
        assert_eq!(name, "my_first_variable");
        data_type.assert(&Some("boolean"));
        default_value.assert(&Some("false"));
    });
}

#[test]
fn simple_not_perfect_definition_parses_correctly() {
    let result = parse!("let my_first_variable := false", statement);
    match_it!(result, Statement::VariableDefinition { name, ref data_type, ref default_value } => {
        assert_eq!(name, "my_first_variable");
        assert_eq!(*data_type, None);
        default_value.assert(&Some("false"));
    });
    let result = parse!("let my_first_variable: boolean", statement);
    match_it!(result, Statement::VariableDefinition { name, ref data_type, ref default_value } => {
        assert_eq!(name, "my_first_variable");
        data_type.assert(&Some("boolean"));
        assert_eq!(*default_value, None);
    });
}

#[test]
fn simple_assignment_parses_correctly() {
    let result = parse!("super_variable := 2 + 2", statement);
    match_it!(result, Statement::VariableAssignment { name, ref expression } => {
        assert_eq!(name, "super_variable");
        expression.assert("2+2");
    });
}

#[test]
fn simple_condition_parses_correctly() {
    let result = parse!("if it.hasNext { it.next }", statement);
    match_it!(result, Statement::Condition { ref condition, ref then_body, ref else_body } => {
        condition.assert("it.hasNext");
        match_it!(&**then_body, &Statement::Expression { ref expression } => {
            expression.assert("it.next()");
        });
        assert_eq!(*else_body, None)
    });
    let result = parse!("if it.hasNext { it.next } else { null }", statement);
    match_it!(result, Statement::Condition { ref condition, ref then_body, ref else_body } => {
        condition.assert("it.hasNext");
        match &**then_body {
            &Statement::Expression { ref expression } => {
                expression.assert("it.next()");
            },
            o => panic!("Pattern Statement::Expression does not match this value {:?}", o),
        }
        match *else_body {
            Some(ref b) => match &**b {
                &Statement::Expression { ref expression } => {
                    expression.assert("null");
                },
                o => panic!("Pattern Statement::Expression does not match this value {:?}", o),
            },
            None => panic!("Option::Some(_) != Option::None"),
        }
    });
}

#[test]
fn simple_simple_cycle_parses_correctly() {
    let result = parse!("loop { 2 + 2 }", statement);
    match_it!(result, Statement::Cycle { ref cycle_type, ref body } => {
        assert_eq!(*cycle_type, CycleType::Simple);
        match_it!(&**body, &Statement::Expression { ref expression } => {
            expression.assert("2 + 2");
        });
    });
}

#[test]
fn simple_while_cycle_parses_correctly() {
    let result = parse!("while true { 2 + 2 }", statement);
    match_it!(result, Statement::Cycle { ref cycle_type, ref body } => {
        match_it!(cycle_type, &CycleType::PrePredicated(ref predicate) => {
            predicate.assert("true");
        });
        match_it!(&**body, &Statement::Expression { ref expression } => {
            expression.assert("2 + 2");
        });
    });
}

#[test]
fn simple_do_while_cycle_parses_correctly() {
    let result = parse!("do { 2 + 2 } while true", statement);
    match_it!(result, Statement::Cycle { ref cycle_type, ref body } => {
        match_it!(cycle_type, &CycleType::PostPredicated(ref predicate) => {
            predicate.assert("true");
        });
        match_it!(&**body, &Statement::Expression { ref expression } => {
            expression.assert("2 + 2");
        });
    });
}

#[test]
fn cycle_control_operators_parses_correctly() {
    let result = parse!("break", statement);
    match_it!(result, Statement::CycleControl { ref operator, ref name } => {
        assert_eq!(*operator, CycleControlOperator::Break);
        assert_eq!(*name, None);
    });
    let result = parse!("break cycle_name", statement);
    match_it!(result, Statement::CycleControl { ref operator, ref name } => {
        assert_eq!(*operator, CycleControlOperator::Break);
        assert_eq!(*name, Some("cycle_name"));
    });
    let result = parse!("continue", statement);
    match_it!(result, Statement::CycleControl { ref operator, ref name } => {
        assert_eq!(*operator, CycleControlOperator::Continue);
        assert_eq!(*name, None);
    });
    let result = parse!("continue cycle_name", statement);
    match_it!(result, Statement::CycleControl { ref operator, ref name } => {
        assert_eq!(*operator, CycleControlOperator::Continue);
        assert_eq!(*name, Some("cycle_name"));
    });
}

#[test]
fn return_operator_parses_correctly() {
    let result = parse!("return", statement);
    match_it!(result, Statement::Return { ref value } => {
        assert_eq!(*value, None);
    });
    let result = parse!("return false", statement);
    match_it!(result, Statement::Return { ref value } => {
        value.assert(&Some("false"));
    });
}

#[test]
fn simple_block_of_statements_parses_correctly() {
    let result = parse!("{ a := 2; return a }", statement);
    match_it!(result, Statement::Block { ref statements } => {
        match_it!(&statements[0], &Statement::VariableAssignment { name, ref expression } => {
            assert_eq!(name, "a");
            expression.assert("2");
        });
        match_it!(&statements[1], &Statement::Return { ref value } => {
            value.assert(&Some("a"));
        });
        assert_eq!(statements.len(), 2);
    });
}

#[test]
fn weird_block_of_statements_parses_correctly() {
    let result = parse!("{}", statement);
    match_it!(result, Statement::Nothing => {});
    let result = parse!("{ return a }", statement);
    match_it!(result, Statement::Return { ref value } => {
        value.assert(&Some("a"));
    });
}

#[test]
fn simple_expression_as_statement_parses_correctly() {
    let result = parse!("a + b * c", statement);
    match_it!(result, Statement::Expression { ref expression } => {
        expression.assert("a + b * c");
    });
}
