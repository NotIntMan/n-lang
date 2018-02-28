use indexmap::IndexMap;
use parser_basics::{
    comma_list,
    identifier,
    keyword,
    symbols,
};
use lexeme_scanner::Token;
use parser_basics::ParserResult;
use desc_lang::compounds::{
    data_type,
    DataType,
};
use man_lang::statements::block;
use super::*;

parser_rule!(type_of(i) -> DataType<'source> {
    do_parse!(i,
        apply!(symbols, ":") >>
        data_type: data_type >>
        (data_type)
    )
});

parser_rule!(argument(i) -> (&'source str, DataType<'source>) {
    do_parse!(i,
        name: identifier >>
        data_type: type_of >>
        ((name, data_type))
    )
});

parser_rule!(arguments(i) -> IndexMap<&'source str, DataType<'source>> {
    do_parse!(i,
        apply!(symbols, "(") >>
        argument_list: apply!(comma_list, argument) >>
        apply!(symbols, ")") >>
        ({
            let mut result = IndexMap::new();
            for (name, data_type) in argument_list {
                result.insert(name, data_type);
            }
            result
        })
    )
});

pub fn function_definition<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, FunctionDefinition<'source>> {
    alt!(input,
        do_parse!(
            apply!(keyword, "extern") >>
            apply!(keyword, "fn") >>
            name: identifier >>
            arguments: arguments >>
            result: opt!(type_of) >>
            (FunctionDefinition {
                name,
                arguments,
                result,
                body: FunctionBody::External,
            })
        )
        | do_parse!(
            apply!(keyword, "fn") >>
            name: identifier >>
            arguments: arguments >>
            result: opt!(type_of) >>
            body: block >>
            (FunctionDefinition {
                name,
                arguments,
                result,
                body: FunctionBody::Implementation(body),
            })
        )
    )
}

#[cfg(test)]
mod tests {
    use helpers::assertion::Assertion;
    use desc_lang::compounds::DataType;
    use desc_lang::functions::{
        FunctionBody,
        function_definition,
        FunctionDefinition,
    };
    use desc_lang::primitives::{
        IntegerType,
        NumberType,
        PrimitiveDataType,
    };
    use man_lang::statements::Statement;
    use man_lang::expressions::{
        BinaryOperator,
        Expression,
    };

    #[test]
    fn simple_external_function_parses_correctly() {
        let result: FunctionDefinition = parse!("extern fn sum(a: integer, b: integer): big integer", function_definition);
        assert_eq!(result.name, "sum");
        let (arg_name, arg_type) = result.arguments.get_index(0)
            .expect("Function's arguments must have the first item");
        assert_eq!(*arg_name, "a");
        arg_type.assert("integer");
        let (arg_name, arg_type) = result.arguments.get_index(1)
            .expect("Function's arguments must have the second item");
        assert_eq!(*arg_name, "b");
        arg_type.assert("integer");
        assert_eq!(result.arguments.get_index(2), None);
        result.result.assert(&Some("big integer"));
        assert_eq!(result.body, FunctionBody::External);
    }

    #[test]
    fn simple_const_time_function_parses_correctly() {
        let result: FunctionDefinition = parse!("\
            fn sum_of_k_series_of_n (k: unsigned integer): unsigned big integer {
                let a := k / 2;
                let b: big integer := k + 1;
                b := a * b;
                return b;
            }
        ", function_definition);
        assert_eq!(result.name, "sum_of_k_series_of_n");
        let (arg_name, arg_type) = result.arguments.get_index(0)
            .expect("Function's arguments must have the first item");
        assert_eq!(*arg_name, "k");
        assert_eq!(*arg_type, DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
            integer_type: IntegerType::Normal,
            unsigned: true,
            zerofill: false,
        })));
        assert_eq!(result.arguments.get_index(1), None);
        assert_eq!(result.result, Some(DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
            integer_type: IntegerType::Big,
            unsigned: true,
            zerofill: false,
        }))));
        let mut statement_iterator = match result.body {
            FunctionBody::Implementation(statement) => match statement {
                Statement::Block { statements } => statements.into_iter(),
                o => panic!("Pattern FunctionBody::Implementation do not matches this value: {:?}", o),
            },
            o => panic!("Pattern FunctionBody::Implementation do not matches this value: {:?}", o),
        };
        let statement = statement_iterator.next()
            .expect("Function's body must have the first statement");
        match_it!(statement, Statement::VariableDefinition { name, data_type, default_value } => {
            assert_eq!(name, "a");
            assert_eq!(data_type, None);
            match_it!(default_value, Some(Expression::BinaryOperation(left, op, right)) => {
                assert_eq!(op, BinaryOperator::Divide);
                left.assert("k");
                right.assert("2");
            });
        });
        let statement = statement_iterator.next()
            .expect("Function's body must have the second statement");
        match_it!(statement, Statement::VariableDefinition { name, data_type, default_value } => {
            assert_eq!(name, "b");
            assert_eq!(data_type, Some(DataType::Primitive(PrimitiveDataType::Number(NumberType::Integer {
                integer_type: IntegerType::Big,
                unsigned: false,
                zerofill: false,
            }))));
            match_it!(default_value, Some(Expression::BinaryOperation(left, op, right)) => {
                assert_eq!(op, BinaryOperator::Plus);
                left.assert("k");
                right.assert("1");
            });
        });
        let statement = statement_iterator.next()
            .expect("Function's body must have the second statement");
        match_it!(statement, Statement::VariableAssignment { name, expression } => {
            assert_eq!(name, "b");
            match_it!(expression, Expression::BinaryOperation(left, op, right) => {
                assert_eq!(op, BinaryOperator::Times);
                left.assert("a");
                right.assert("b");
            });
        });
        let statement = statement_iterator.next()
            .expect("Function's body must have the second statement");
        match_it!(statement, Statement::Return { value } => {
            value.assert(&Some("b"));
        });
        assert_eq!(statement_iterator.next(), None);
    }
}
