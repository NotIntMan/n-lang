use lexeme_scanner::Token;
use parser_basics::{
    Parser,
    keyword,
    ParserResult,
    symbols,
};
use language::expressions::{
    BinaryOperator,
    ExpressionAST,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolutionOrder {
    Left,
    Right,
}

fn fold_left<'source>(
    operator: BinaryOperator,
    left: ExpressionAST<'source>,
    tails: Vec<ExpressionAST<'source>>,
) -> ExpressionAST<'source> {
    let mut result = left;
    for tail in tails {
        result = ExpressionAST::BinaryOperation(Box::new(result), operator, Box::new(tail));
    }
    result
}

type Resolver<'a, 'b> = (ResolutionOrder, BinaryOperator, Parser<'a, 'b, ()>);

fn infix<'token, 'source>(
    input: &'token [Token<'source>],
    resolvers: &[Resolver<'token, 'source>],
    atom: Parser<'token, 'source, ExpressionAST<'source>>,
) -> ParserResult<'token, 'source, ExpressionAST<'source>> {
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
                    Some(tail) => ExpressionAST::BinaryOperation(Box::new(left), operator, Box::new(tail)),
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
/// "="
parser_rule!(equals(i) -> () { do_parse!(i, apply!(symbols, "=") >> (())) });
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
/// "regexp"
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
    atom: Parser<'token, 'source, ExpressionAST<'source>>,
) -> ParserResult<'token, 'source, ExpressionAST<'source>> {
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
