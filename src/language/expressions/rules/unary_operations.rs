use lexeme_scanner::{
    ItemPosition,
    SymbolPosition,
    Token,
};
use parser_basics::{
    keyword,
    Parser,
    ParserResult,
    symbol_position,
    symbols,
};
use super::super::*;

/// "!"
parser_rule!(logic_not(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(symbols, "!") >> (PrefixUnaryOperator::Not))
});
/// "all"
parser_rule!(all(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(keyword, "all") >> (PrefixUnaryOperator::All))
});
/// "any"
parser_rule!(any(i) -> PrefixUnaryOperator {
    do_parse!(i, apply!(keyword, "any") >> (PrefixUnaryOperator::Any))
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
pub fn prefix_unary_operator<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, (PrefixUnaryOperator, SymbolPosition)> {
    do_parse!(input,
        begin: symbol_position >>
        operator: alt!(
            logic_not |
            all |
            any |
            plus |
            minus |
            tilde |
            binary |
            row |
            exists
        ) >>
        ((operator, begin))
    )
}

/// Создаёт новое `Expression::PrefixUnaryOperation`, помещая в него данный оператор и упакованное данное выражение
#[inline]
pub fn make_prefix_unary<'source>(operator: PrefixUnaryOperator, begin: SymbolPosition, expr: ExpressionAST<'source>) -> ExpressionAST<'source> {
    let pos = ItemPosition {
        begin,
        end: expr.pos.end,
    };
    ExpressionAST {
        body: ExpressionASTBody::PrefixUnaryOperation(operator, Box::new(expr)),
        pos,
    }
}

#[inline]
pub fn fold_prefix_unary<'source>(mut operators: Vec<(PrefixUnaryOperator, SymbolPosition)>, mut expression: ExpressionAST<'source>) -> ExpressionAST<'source> {
    while let Some((operator, begin_position)) = operators.pop() {
        expression = make_prefix_unary(operator, begin_position, expression);
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
pub fn postfix_unary_operation<'token, 'source>(input: &'token [Token<'source>], expr: ExpressionAST<'source>) -> ParserResult<'token, 'source, ExpressionAST<'source>> {
    do_parse!(input,
        items: many0!(do_parse!(
            apply!(keyword, "is") >>
            not: is_not >>
            what: is_what >>
            end: symbol_position >>
            ((not, what, end))
        )) >>
        ({
            let mut result = expr;
            for (not, what, end) in items {
                let pos = ItemPosition {
                    begin: result.pos.begin,
                    end,
                };
                result = ExpressionAST {
                    body: ExpressionASTBody::PostfixUnaryOperation(what, Box::new(result)),
                    pos,
                };
                if not {
                    result = make_prefix_unary(PrefixUnaryOperator::Not, result.pos.begin, result)
                }
            }
            result
        })
    )
}

pub fn unary_operation<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, ExpressionAST<'source>>,
) -> ParserResult<'token, 'source, ExpressionAST<'source>> {
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
