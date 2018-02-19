extern crate n_transpiler;

use n_transpiler::lexeme_scanner::Scanner;
use n_transpiler::parser_basics::parse;
use n_transpiler::man_lang::expressions::*;

fn extract_literal<'source>(expr: Expression<'source>) -> LiteralType {
    match expr {
        Expression::Literal(lit) => lit.literal_type,
        e => panic!("This is not literal: {:?}", e),
    }
}

#[test]
fn number_literals_parses_correctly() {
    {
        let tokens = Scanner::scan("-2")
            .expect("Scan result with no error");
        assert_eq!(
            extract_literal(parse(tokens.as_slice(), expression)
                .expect("Parse result with no error")),
            LiteralType::NumberLiteral {
                negative: true,
                fractional: false,
                radix: 10,
            }
        );
    }
    {
        let tokens = Scanner::scan("0b101.1")
            .expect("Scan result with no error");
        assert_eq!(
            extract_literal(parse(tokens.as_slice(), expression)
                .expect("Parse result with no error")),
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
        let tokens = Scanner::scan("\"azaz\"")
            .expect("Scan result with no error");
        assert_eq!(
            extract_literal(parse(tokens.as_slice(), expression)
                .expect("Parse result with no error")),
            LiteralType::StringLiteral { length: 4 }
        );
    }
    {
        let tokens = Scanner::scan("'can\nzas'")
            .expect("Scan result with no error");
        assert_eq!(
            extract_literal(parse(tokens.as_slice(), expression)
                .expect("Parse result with no error")),
            LiteralType::BracedExpressionLiteral { length: 7 }
        );
    }
}

#[test]
fn keyword_literals_parses_correctly() {
    {
        let tokens = Scanner::scan("true")
            .expect("Scan result with no error");
        assert_eq!(
            extract_literal(parse(tokens.as_slice(), expression)
                .expect("Parse result with no error")),
            LiteralType::KeywordLiteral(KeywordLiteralType::True)
        );
    }
    {
        let tokens = Scanner::scan("false")
            .expect("Scan result with no error");
        assert_eq!(
            extract_literal(parse(tokens.as_slice(), expression)
                .expect("Parse result with no error")),
            LiteralType::KeywordLiteral(KeywordLiteralType::False)
        );
    }
    {
        let tokens = Scanner::scan("null")
            .expect("Scan result with no error");
        assert_eq!(
            extract_literal(parse(tokens.as_slice(), expression)
                .expect("Parse result with no error")),
            LiteralType::KeywordLiteral(KeywordLiteralType::Null)
        );
    }
}

fn assert_identifier<'source>(expr: Expression, text: &str) {
    match expr {
        Expression::Identifier(token) => assert_eq!(token.text, text),
        e => panic!("This is not binary identifier: {:?}", e),
    }
}

fn extract_prefix_unary_expression<'source>(expr: Expression<'source>, operator: PrefixUnaryOperator) -> Expression<'source> {
    match expr {
        Expression::PrefixUnaryOperation(op, expr) => {
            assert_eq!(op, operator);
            *expr
        },
        e => panic!("This is not prefix unary operation {:?}", e),
    }
}
fn extract_postfix_unary_expression<'source>(expr: Expression<'source>, operator: PostfixUnaryOperator) -> Expression<'source> {
    match expr {
        Expression::PostfixUnaryOperation(op, expr) => {
            assert_eq!(op, operator);
            *expr
        },
        e => panic!("This is not postfix unary operation {:?}", e),
    }
}

#[test]
fn all_unary_operations_parses_correctly() {
    let tokens = Scanner::scan(
        "!all any + - ~ binary row exists \
        data \
        is unknown is false is not unknown is not false is true is not true is not null is null"
    ).expect("Scan result must be ok");
    let mut expr = parse(tokens.as_slice(), expression)
        .expect("Parse result must be ok");
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
    match expr {
        Expression::BinaryOperation(left, op, right) => {
            assert_eq!(op, operator);
            (*left, *right)
        },
        e => panic!("This is not binary expression: {:?}", e),
    }
}

#[test]
fn simple_infix_expression_parses_correctly() {
    let tokens = Scanner::scan("a and b or c xor d")
        .expect("Scan result must be ok");
    let result = parse(tokens.as_slice(), expression)
        .expect("Parse result must be ok");
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
    let tokens = Scanner::scan("a and b or c xor d or e ** f")
        .expect("Scan result must be ok");
    let result = parse(tokens.as_slice(), expression)
        .expect("Parse result must be ok");
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
        let tokens = Scanner::scan(&input)
            .expect("Scan result must be ok");
        let result = parse(tokens.as_slice(), expression)
            .expect("Parse result must be ok");
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
    assert_operation("==", BinaryOperator::Equals);
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

fn extract_property_access_expression<'source>(expr: Expression<'source>, property_name: &str) -> Expression<'source> {
    match expr {
        Expression::PropertyAccess(expr, prop) => {
            assert_eq!(prop, property_name);
            *expr
        },
        e => panic!("This is not property access expression {:?}", e),
    }
}
fn extract_set_expression<'source>(expr: Expression<'source>) -> Vec<Expression<'source>> {
    match expr {
        Expression::Set(vec) => vec,
        e => panic!("This is not set expression {:?}", e),
    }
}
fn extract_function_call_expression<'source>(expr: Expression<'source>, function_name: &str) -> Vec<Expression<'source>> {
    match expr {
        Expression::FunctionCall(name, args) => {
            assert_eq!(name, function_name);
            args
        },
        e => panic!("This is not function call expression {:?}", e),
    }
}

#[test]
fn property_access_and_set_and_function_call_expression_parses_correctly() {
    let tokens = Scanner::scan(
        "foo(bar, bar.baz, (box, boz))"
    ).expect("Scan result must be ok");
    let result = parse(tokens.as_slice(), expression)
        .expect("Parse result must be ok");
    let mut args = extract_function_call_expression(result, "foo");
    assert_identifier(args.remove(0), "bar");
    let arg1 = extract_property_access_expression(args.remove(0), "baz");
    assert_identifier(arg1, "bar");
    let mut arg2 = extract_set_expression(args.remove(0));
    assert_identifier(arg2.remove(0), "box");
    assert_identifier(arg2.remove(0), "boz");
}
