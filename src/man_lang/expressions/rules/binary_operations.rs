use lexeme_scanner::Token;
use parser_basics::{
    Parser,
    keyword,
    ParserResult,
    symbols,
};
use man_lang::expressions::{
    BinaryOperator,
    Expression,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolutionOrder {
    Left,
    Right,
}

fn fold_left<'source>(
    operator: BinaryOperator,
    left: Expression<'source>,
    tails: Vec<Expression<'source>>,
) -> Expression<'source> {
    let mut result = left;
    for tail in tails {
        result = Expression::BinaryOperation(Box::new(result), operator, Box::new(tail));
    }
    result
}

type Resolver<'a, 'b> = (ResolutionOrder, BinaryOperator, Parser<'a, 'b, ()>);

fn infix<'token, 'source>(
    input: &'token [Token<'source>],
    resolvers: &[Resolver<'token, 'source>],
    atom: Parser<'token, 'source, Expression<'source>>,
) -> ParserResult<'token, 'source, Expression<'source>> {
    if resolvers.is_empty() {
        return atom(input);
    }
    let (
        order,
        operator,
        operation,
    ) = resolvers[0].clone();
    match order {
        ResolutionOrder::Left => {
            do_parse!(input,
                left: apply!(infix, &resolvers[1..], atom) >>
                tails: many0!(do_parse!(
                    operation >>
                    right: apply!(infix, &resolvers[1..], atom) >>
                    (right)
                )) >>
                (fold_left(operator, left, tails))
            )
        },
        ResolutionOrder::Right => {
            do_parse!(input,
                left: apply!(infix, &resolvers[1..], atom) >>
                tail: opt!(do_parse!(
                    operation >>
                    right: apply!(infix, resolvers, atom) >>
                    (right)
                )) >>
                (match tail {
                    Some(tail) => Expression::BinaryOperation(Box::new(left), operator, Box::new(tail)),
                    None => left,
                })
            )
        },
    }
}

/// "or" | "||"
parser_rule!(logic_or(i) -> () {
    alt!(i, apply!(keyword, "or") => {|_| ()} | apply!(symbols, "||") => {|_| ()})
});
/// "xor" | "^^"
parser_rule!(logic_xor(i) -> () {
    alt!(i, apply!(keyword, "xor") => {|_| ()} | apply!(symbols, "^^") => {|_| ()})
});
/// "and" | "&&"
parser_rule!(logic_and(i) -> () {
    alt!(i, apply!(keyword, "and") => {|_| ()} | apply!(symbols, "&&") => {|_| ()})
});
/// "|"
parser_rule!(bit_or(i) -> () { do_parse!(i, apply!(symbols, "|") >> (())) });
/// "^"
parser_rule!(bit_xor(i) -> () { do_parse!(i, apply!(symbols, "^") >> (())) });
/// "&"
parser_rule!(bit_and(i) -> () { do_parse!(i, apply!(symbols, "&") >> (())) });
/// "<<"
parser_rule!(shift_left(i) -> () { do_parse!(i, apply!(symbols, "<<") >> (())) });
/// ">>"
parser_rule!(shift_right(i) -> () { do_parse!(i, apply!(symbols, ">>") >> (())) });
/// "is" "in"
parser_rule!(is_in(i) -> () { do_parse!(i, apply!(keyword, "is") >> apply!(keyword, "in") >> (())) });
/// "=="
parser_rule!(equals(i) -> () { do_parse!(i, apply!(symbols, "==") >> (())) });
/// ">="
parser_rule!(more_than_or_equals(i) -> () { do_parse!(i, apply!(symbols, ">=") >> (())) });
/// ">"
parser_rule!(more_than(i) -> () { do_parse!(i, apply!(symbols, ">") >> (())) });
/// "<="
parser_rule!(less_than_or_equals(i) -> () { do_parse!(i, apply!(symbols, "<=") >> (())) });
/// "<"
parser_rule!(less_than(i) -> () { do_parse!(i, apply!(symbols, "<") >> (())) });
/// "like"
parser_rule!(like(i) -> () { do_parse!(i, apply!(keyword, "like") >> (())) });
/// "sounds" "like"
parser_rule!(sounds_like(i) -> () { do_parse!(i, apply!(keyword, "sounds") >> apply!(keyword, "like") >> (())) });
/// regexp"
parser_rule!(reg_exp(i) -> () { do_parse!(i, apply!(keyword, "regexp") >> (())) });
/// "+"
parser_rule!(arithmetic_plus(i) -> () { do_parse!(i, apply!(symbols, "+") >> (())) });
/// "-"
parser_rule!(arithmetic_minus(i) -> () { do_parse!(i, apply!(symbols, "-") >> (())) });
/// "*"
parser_rule!(arithmetic_times(i) -> () { do_parse!(i, apply!(symbols, "*") >> (())) });
/// "/"
parser_rule!(arithmetic_divide(i) -> () { do_parse!(i, apply!(symbols, "/") >> (())) });
/// "mod" | "%"
parser_rule!(arithmetic_mod(i) -> () {
    alt!(i, apply!(keyword, "mod") => {|_| ()} | apply!(symbols, "%") => {|_| ()})
});
/// "div"
parser_rule!(arithmetic_div(i) -> () { do_parse!(i, apply!(keyword, "div") >> (())) });
/// "**"
parser_rule!(arithmetic_pow(i) -> () { do_parse!(i, apply!(symbols, "**") >> (())) });
/// ".."
parser_rule!(interval(i) -> () { do_parse!(i, apply!(symbols, "..") >> (())) });

pub fn binary_expression<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, Expression<'source>>,
) -> ParserResult<'token, 'source, Expression<'source>> {
    array!(let resolvers: Resolver<'token, 'source> =
        (ResolutionOrder::Left, BinaryOperator::Or, logic_or),
        (ResolutionOrder::Left, BinaryOperator::XOr, logic_xor),
        (ResolutionOrder::Left, BinaryOperator::And, logic_and),
        (ResolutionOrder::Left, BinaryOperator::BitOr, bit_or),
        (ResolutionOrder::Left, BinaryOperator::BitXOr, bit_xor),
        (ResolutionOrder::Left, BinaryOperator::BitAnd, bit_and),
        (ResolutionOrder::Left, BinaryOperator::ShiftLeft, shift_left),
        (ResolutionOrder::Left, BinaryOperator::ShiftRight, shift_right),
        (ResolutionOrder::Left, BinaryOperator::IsIn, is_in),
        (ResolutionOrder::Left, BinaryOperator::Equals, equals),
        (ResolutionOrder::Left, BinaryOperator::MoreThanOrEquals, more_than_or_equals),
        (ResolutionOrder::Left, BinaryOperator::MoreThan, more_than),
        (ResolutionOrder::Left, BinaryOperator::LessThanOrEquals, less_than_or_equals),
        (ResolutionOrder::Left, BinaryOperator::LessThan, less_than),
        (ResolutionOrder::Left, BinaryOperator::Like, like),
        (ResolutionOrder::Left, BinaryOperator::SoundsLike, sounds_like),
        (ResolutionOrder::Left, BinaryOperator::RegExp, reg_exp),
        (ResolutionOrder::Left, BinaryOperator::Plus, arithmetic_plus),
        (ResolutionOrder::Left, BinaryOperator::Minus, arithmetic_minus),
        (ResolutionOrder::Left, BinaryOperator::Times, arithmetic_times),
        (ResolutionOrder::Left, BinaryOperator::Divide, arithmetic_divide),
        (ResolutionOrder::Left, BinaryOperator::Mod, arithmetic_mod),
        (ResolutionOrder::Left, BinaryOperator::Div, arithmetic_div),
        (ResolutionOrder::Right, BinaryOperator::Pow, arithmetic_pow),
        (ResolutionOrder::Left, BinaryOperator::Interval, interval),
    );
    infix(input, &resolvers[..], atom)
}

#[test]
fn simple_infix_expression_parses_correctly() {
    let extract_binary_expression = |expr|  match expr {
        Expression::BinaryOperation(left, op, right) => (*left, op, *right),
        e => panic!("This is not binary operation {:?}", e),
    };
    let extract_identifier = |expr| match expr {
        Expression::Identifier(t) => t.text,
        e => panic!("This is not identifier {:?}", e),
    };
    use super::expression;
    parser_rule!(bin(i) -> Expression<'source> { binary_expression(i, expression) });
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
    let tokens = Scanner::scan("a and b or c xor d")
        .expect("Scan result must be ok");
    let result = parse(tokens.as_slice(), bin)
        .expect("Parse result must be ok");
    let (left, op, right) = extract_binary_expression(result);
    assert_eq!(op, BinaryOperator::Or);
    let (left_left, left_op, left_right) = extract_binary_expression(left);
    assert_eq!(left_op, BinaryOperator::And);
    assert_eq!(extract_identifier(left_left), "a");
    assert_eq!(extract_identifier(left_right), "b");
    let (right_left, right_op, right_right) = extract_binary_expression(right);
    assert_eq!(right_op, BinaryOperator::XOr);
    assert_eq!(extract_identifier(right_left), "c");
    assert_eq!(extract_identifier(right_right), "d");
}

#[test]
fn complex_infix_expression_parses_correctly() {
    let extract_binary_expression = |expr|  match expr {
        Expression::BinaryOperation(left, op, right) => (*left, op, *right),
        e => panic!("This is not binary operation {:?}", e),
    };
    let extract_identifier = |expr| match expr {
        Expression::Identifier(t) => t.text,
        e => panic!("This is not identifier {:?}", e),
    };
    use super::expression;
    parser_rule!(bin(i) -> Expression<'source> { binary_expression(i, expression) });
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
    let tokens = Scanner::scan("a and b or c xor d or e ** f")
        .expect("Scan result must be ok");
    let result = parse(tokens.as_slice(), bin)
        .expect("Parse result must be ok");
    let (left, op, right) = extract_binary_expression(result);
    assert_eq!(op, BinaryOperator::Or);
    {
        let (left, op, right) = extract_binary_expression(left);
        assert_eq!(op, BinaryOperator::Or);
        {
            let (left, op, right) = extract_binary_expression(left);
            assert_eq!(op, BinaryOperator::And);
            assert_eq!(extract_identifier(left), "a");
            assert_eq!(extract_identifier(right), "b");
        }
        {
            let (left, op, right) = extract_binary_expression(right);
            assert_eq!(op, BinaryOperator::XOr);
            assert_eq!(extract_identifier(left), "c");
            assert_eq!(extract_identifier(right), "d");
        }
    }
    {
        let (left, op, right) = extract_binary_expression(right);
        assert_eq!(op, BinaryOperator::Pow);
        assert_eq!(extract_identifier(left), "e");
        assert_eq!(extract_identifier(right), "f");
    }
}

#[test]
fn simple_operations_of_all_types_parses_correctly() {
    fn extract_binary_expression<'source>(expr: Expression<'source>) -> (Expression<'source>, BinaryOperator, Expression<'source>) {
        match expr {
            Expression::BinaryOperation(left, op, right) => (*left, op, *right),
            e => panic!("This is not binary operation {:?}", e.clone()),
        }
    }
    fn extract_identifier<'source>(expr: Expression<'source>) -> &'source str {
        match expr {
            Expression::Identifier(t) => t.text,
            e => panic!("This is not identifier {:?}", e.clone()),
        }
    }
    use super::expression;
    parser_rule!(bin(i) -> Expression<'source> { binary_expression(i, expression) });
    fn assert_operation(text: &str, operator: BinaryOperator) {
        use lexeme_scanner::Scanner;
        use parser_basics::parse;
        let input = format!("left_identifier {} RightIdentifier", text);
        let tokens = Scanner::scan(&input)
            .expect("Scan result must be ok");
        let result = parse(tokens.as_slice(), bin)
            .expect("Parse result must be ok");
        let (left, op, right) = extract_binary_expression(result);
        assert_eq!(op, operator);
        assert_eq!(extract_identifier(left), "left_identifier");
        assert_eq!(extract_identifier(right), "RightIdentifier");
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
