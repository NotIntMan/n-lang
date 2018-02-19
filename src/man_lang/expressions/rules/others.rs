use parser_basics::{
    comma_list,
    identifier,
    Parser,
    ParserResult,
    symbols,
};

use super::*;

pub fn property_access<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, Expression<'source>>,
) -> ParserResult<'token, 'source, Expression<'source>> {
    do_parse!(input,
        atomic: atom >>
        tail: opt!(do_parse!(
            apply!(symbols, ".") >>
            name: identifier >>
            (name)
        )) >>
        (match tail {
            Some(name) => Expression::PropertyAccess(Box::new(atomic), name),
            None => atomic,
        })
    )
}

parser_rule!(expression_set(i, atom: Parser<'token, 'source, Expression<'source>>) -> Vec<Expression<'source>> {
    do_parse!(i,
        apply!(symbols, "(") >>
        items: apply!(comma_list, atom) >>
        apply!(symbols, ")") >>
        (items)
    )
});

pub fn set<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, Expression<'source>>,
) -> ParserResult<'token, 'source, Expression<'source>> {
    expression_set(input, atom)
        .map(|mut items| if items.len() == 1 {
            items.swap_remove(0)
        } else {
            Expression::Set(items)
        })
}

pub fn function_call<'token, 'source>(
    input: &'token [Token<'source>],
    atom: Parser<'token, 'source, Expression<'source>>,
) -> ParserResult<'token, 'source, Expression<'source>> {
    do_parse!(input,
        name: identifier >>
        args: apply!(expression_set, atom) >>
        (Expression::FunctionCall(name, args))
    )
}

#[test]
fn property_access_and_set_and_function_call_expression_parses_correctly() {
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
    fn assert_identifier<'source>(expr: Expression<'source>, text: &str) {
        match expr {
            Expression::Identifier(t) => assert_eq!(t.text, text),
            e => panic!("This is not identifier {:?}", e),
        }
    }
    use super::expression;
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
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
