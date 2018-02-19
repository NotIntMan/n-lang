use lexeme_scanner::Token;
use parser_basics::{
    keyword,
    Parser,
    ParserResult,
    symbols,
};
use super::super::*;

/// "!"
parser_rule!(logic_not(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(symbols, "!") >> (PrefixUnaryOperator::Not))
});
/// "all"
parser_rule!(all(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(keyword, "all") >> (PrefixUnaryOperator::Not))
});
/// "any"
parser_rule!(any(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(keyword, "any") >> (PrefixUnaryOperator::Not))
});
/// "+"
parser_rule!(plus(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(symbols, "+") >> (PrefixUnaryOperator::Plus))
});
/// "-"
parser_rule!(minus(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(symbols, "-") >> (PrefixUnaryOperator::Minus))
});
/// "~"
parser_rule!(tilde(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(symbols, "~") >> (PrefixUnaryOperator::Tilde))
});
/// "binary"
parser_rule!(binary(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(keyword, "binary") >> (PrefixUnaryOperator::Binary))
});
/// "row"
parser_rule!(row(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(keyword, "row") >> (PrefixUnaryOperator::Row))
});
/// "exists"
parser_rule!(exists(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(keyword, "exists") >> (PrefixUnaryOperator::Exists))
});

/// Функция, выполняющая разбор префиксного унарного оператора инфиксных выражений
pub fn prefix_unary_operator<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, PrefixUnaryOperator> {
    alt!(input,
        logic_not |
        all |
        any |
        plus |
        minus |
        tilde |
        binary |
        row |
        exists
    )
}

/// Создаёт новое `Expression::PrefixUnaryOperation`, помещая в него данный оператор и упакованное данное выражение
#[inline]
pub fn make_prefix_unary<'source>(operator: PrefixUnaryOperator, expr: Expression<'source>) -> Expression<'source> {
    Expression::PrefixUnaryOperation(operator, Box::new(expr))
}

#[inline]
pub fn fold_prefix_unary<'source>(mut operators: Vec<PrefixUnaryOperator>, mut expression: Expression<'source>) -> Expression<'source> {
    while let Some(operator) = operators.pop() {
        expression = make_prefix_unary(operator, expression);
    }
    expression
}

/// ["not"]
parser_rule!(is_not(i) -> bool {
    do_parse!(i,
        x: opt!(apply!(keyword, "not")) >>
        (x.is_some())
    )
});

/// "null" | "true" | "false" | "unknown"
parser_rule!(is_what(i) -> PostfixUnaryOperator {
    alt!(i,
        apply!(keyword, "null") => { |_| PostfixUnaryOperator::IsNull } |
        apply!(keyword, "true") => { |_| PostfixUnaryOperator::IsTrue } |
        apply!(keyword, "false") => { |_| PostfixUnaryOperator::IsFalse } |
        apply!(keyword, "unknown") => { |_| PostfixUnaryOperator::IsUnknown }
    )
});

/// Функция, выполняющая разбор постфиксного унарного оператора инфиксных выражений.
/// При необходимости, оборачивает его в префиксное `not`.
/// В случае неудачи разбора операций, возвращает переданное выражение.
pub fn postfix_unary_operation<'token, 'source>(input: &'token [Token<'source>], expr: Expression<'source>) -> ParserResult<'token, 'source, Expression<'source>> {
    do_parse!(input,
        apply!(keyword, "is") >>
        items: many0!(do_parse!(
            not: is_not >>
            what: is_what >>
            ((not, what))
        )) >>
        ({
            let mut result = expr;
            for (not, what) in items {
                result = Expression::PostfixUnaryOperation(what, Box::new(result));
                if not {
                    result = make_prefix_unary(PrefixUnaryOperator::Not, result)
                }
            }
            result
        })
    )
}

pub fn unary_operation<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, Expression<'source>>,
) -> ParserResult<'token, 'source, Expression<'source>> {
    do_parse!(input,
        expr: do_parse!(
            prefix: many0!(prefix_unary_operator) >>
            atomic: atom >>
            (fold_prefix_unary(prefix, atomic))
        ) >>
        result: apply!(postfix_unary_operation, expr) >>
        (result)
    )
}
