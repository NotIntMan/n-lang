use lexeme_scanner::Token;
use parser_basics::{
    Parser,
    keyword,
    ParserResult,
};
use man_lang::expressions::{
    BinaryOperator,
    Expression,
    expression,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolutionOrder {
    Left,
    Right,
}

// TODO Посмотреть, не удалить ли из этой функции вариант левосторонней свёртки
fn fold_operation<'source>(
    order: ResolutionOrder,
    operator: BinaryOperator,
    left: Expression<'source>,
    mut tails: Vec<Expression<'source>>,
) -> Expression<'source> {
    match order {
        ResolutionOrder::Left => {
            let mut result = left;
            for tail in tails {
                result = Expression::BinaryOperation(Box::new(result), operator, Box::new(tail));
            }
            result
        },
        ResolutionOrder::Right => {
            match tails.pop() {
                None => left,
                Some(tail) => {
                    let mut result = tail;
                    while let Some(tail) = tails.pop() {
                        result = Expression::BinaryOperation(Box::new(result), operator, Box::new(tail));
                    }
                    Expression::BinaryOperation(Box::new(left), operator, Box::new(result))
                },
            }
        },
    }
}

type Resolver<'a, 'b> = (ResolutionOrder, BinaryOperator, Parser<'a, 'b, ()>);

fn infix<'token, 'source>(input: &'token [Token<'source>], resolvers: &[Resolver<'token, 'source>]) -> ParserResult<'token, 'source, Expression<'source>> {
    if resolvers.is_empty() {
        return expression(input);
    }
    let (
        order,
        operator,
        operation,
    ) = resolvers[0].clone();
    match order {
        ResolutionOrder::Left => {
            do_parse!(input,
                left: apply!(infix, &resolvers[1..]) >>
                tail: opt!(do_parse!(
                    operation >>
                    right: apply!(infix, resolvers) >>
                    (right)
                )) >>
                (match tail {
                    Some(tail) => Expression::BinaryOperation(Box::new(left), operator, Box::new(tail)),
                    None => left,
                })
            )
        },
        ResolutionOrder::Right => {
            do_parse!(input,
                left: apply!(infix, &resolvers[1..]) >>
                tails: many0!(do_parse!(
                    operation >>
                    right: apply!(infix, &resolvers[1..]) >>
                    (right)
                )) >>
                (fold_operation(order, operator, left, tails))
            )
        },
    }
}

parser_rule!(logic_or(i) -> () { do_parse!(i, apply!(keyword, "or") >> (())) });
parser_rule!(logic_xor(i) -> () { do_parse!(i, apply!(keyword, "xor") >> (())) });
parser_rule!(logic_and(i) -> () { do_parse!(i, apply!(keyword, "and") >> (())) });

pub fn binary_expression<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Expression<'source>> {
    array!(let resolvers: Resolver<'token, 'source> =
        (ResolutionOrder::Left, BinaryOperator::Or, logic_or),
        (ResolutionOrder::Left, BinaryOperator::XOr, logic_xor),
        (ResolutionOrder::Left, BinaryOperator::And, logic_and),
    );
    infix(input, &resolvers[..])
}

#[test]
fn simple_infix_expression_parses_correctly() {
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
    let tokens = Scanner::scan("a and b or c xor d")
        .expect("Scan result must be ok");
    let result = parse(tokens.as_slice(), binary_expression)
        .expect("Parse result must be ok");
    let (left, op, right) = match result {
        Expression::BinaryOperation(left, op, right) => (*left, op, *right),
        e => panic!("This is not binary operation {:?}", e),
    };
    assert_eq!(op, BinaryOperator::Or);
    let (left_left, left_op, left_right) = match left {
        Expression::BinaryOperation(left, op, right) => (*left, op, *right),
        e => panic!("This is not binary operation {:?}", e),
    };
    assert_eq!(left_op, BinaryOperator::And);
    match left_left {
        Expression::Identifier(t) => assert_eq!(t.text, "a"),
        e => panic!("This is not identifier {:?}", e),
    }
    match left_right {
        Expression::Identifier(t) => assert_eq!(t.text, "b"),
        e => panic!("This is not identifier {:?}", e),
    }
    let (right_left, right_op, right_right) = match right {
        Expression::BinaryOperation(left, op, right) => (*left, op, *right),
        e => panic!("This is not binary operation {:?}", e),
    };
    assert_eq!(right_op, BinaryOperator::XOr);
    match right_left {
        Expression::Identifier(t) => assert_eq!(t.text, "c"),
        e => panic!("This is not identifier {:?}", e),
    }
    match right_right {
        Expression::Identifier(t) => assert_eq!(t.text, "d"),
        e => panic!("This is not identifier {:?}", e),
    }
}
