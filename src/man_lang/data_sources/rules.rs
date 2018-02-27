use lexeme_scanner::Token;
use parser_basics::{
    comma_list,
    identifier,
    keyword,
    not_keyword_identifier,
    ParserResult,
    symbols,
};
use man_lang::expressions::expression;
use man_lang::others::property_path;
use man_lang::selections::selection;
use super::*;

parser_rule!(join_condition(i) -> JoinCondition<'source> {
    alt!(i,
        do_parse!(
            apply!(keyword, "on") >>
            x: expression >>
            (JoinCondition::Expression(x))
        )
        | do_parse!(
            apply!(keyword, "using") >>
            apply!(symbols, "(") >>
            fields: apply!(comma_list, property_path) >>
            apply!(symbols, ")") >>
            (JoinCondition::Using(fields))
        )
    )
});

parser_rule!(table(i) -> DataSource<'source> {
    do_parse!(i,
        name: identifier >>
        alias: opt!(not_keyword_identifier) >>
        (DataSource::Table { name, alias })
    )
});

parser_rule!(join_source(i) -> DataSource<'source> {
    alt!(i,
        table
        | do_parse!(
            apply!(symbols, "(") >>
            source: data_source >>
            apply!(symbols, ")") >>
            (source)
        )
        | do_parse!(
            apply!(symbols, "(") >>
            query: selection >>
            apply!(symbols, ")") >>
            opt!(apply!(keyword, "as")) >>
            alias: not_keyword_identifier >>
            (DataSource::Selection { query: Box::new(query), alias })
        )
    )
});

type JoinTail<'source> = (JoinType, Option<JoinCondition<'source>>, DataSource<'source>);
parser_rule!(join_tail(i) -> JoinTail<'source> {
    alt!(i,
        do_parse!(
            apply!(keyword, "natural") >>
            apply!(keyword, "left") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            source: join_source >>
            ((JoinType::Left, Some(JoinCondition::Natural), source))
        )
        | do_parse!(
            apply!(keyword, "left") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            source: join_source >>
            condition: opt!(join_condition) >>
            ((JoinType::Left, condition, source))
        )
        | do_parse!(
            apply!(keyword, "natural") >>
            apply!(keyword, "right") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            source: join_source >>
            ((JoinType::Right, Some(JoinCondition::Natural), source))
        )
        | do_parse!(
            apply!(keyword, "right") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            source: join_source >>
            condition: opt!(join_condition) >>
            ((JoinType::Right, condition, source))
        )
        | do_parse!(
            apply!(keyword, "natural") >>
            apply!(keyword, "full") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            source: join_source >>
            ((JoinType::Full, Some(JoinCondition::Natural), source))
        )
        | do_parse!(
            apply!(keyword, "full") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            source: join_source >>
            condition: opt!(join_condition) >>
            ((JoinType::Full, condition, source))
        )
        | do_parse!(
            apply!(keyword, "inner") >>
            apply!(keyword, "join") >>
            source: join_source >>
            condition: opt!(join_condition) >>
            ((JoinType::Cross, condition, source))
        )
        | do_parse!(
            opt!(apply!(keyword, "cross")) >>
            apply!(keyword, "join") >>
            source: join_source >>
            ((JoinType::Cross, None, source))
        )
        | do_parse!(
            apply!(symbols, ",") >>
            source: join_source >>
            ((JoinType::Cross, None, source))
        )
    )
});

fn fold_join<'source>(mut origin: DataSource<'source>, tails: Vec<JoinTail<'source>>) -> DataSource<'source> {
    for (join_type, condition, right) in tails {
        origin = DataSource::Join {
            join_type,
            condition,
            left: Box::new(origin),
            right: Box::new(right),
        };
    }
    origin
}

/// Функция, выполняющая разбор источника данных запроса (таблиц и их объединений)
pub fn data_source<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, DataSource<'source>> {
    do_parse!(input,
        origin: alt!(join_source) >>
        tails: many0!(join_tail) >>
        (fold_join(origin, tails))
    )
}

#[cfg(test)]
mod tests {
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
    use man_lang::expressions::expression;
    use man_lang::data_sources::{
        data_source,
        DataSource,
        JoinCondition,
        JoinType,
    };

    fn assert_table(source: DataSource, table_name: &str, table_alias: Option<&str>) {
        match source {
            DataSource::Table { name, alias } => {
                assert_eq!(name, table_name);
                assert_eq!(alias, table_alias);
            },
            o => panic!("This is not table {:?}", o),
        }
    }

    fn extract_join<'source>(source: DataSource<'source>, expected_type: JoinType, expected_condition: Option<JoinCondition>) -> (DataSource<'source>, DataSource<'source>) {
        match source {
            DataSource::Join { join_type, condition, left, right } => {
                assert_eq!(join_type, expected_type);
                assert_eq!(condition, expected_condition);
                (*left, *right)
            },
            o => panic!("This is not join {:?}", o),
        }
    }

    #[test]
    fn simple_join_parses_correctly() {
        let tokens = Scanner::scan("foo f INNER JOIN bar b ON f.foo_id = b.bar_id")
            .expect("Scanner result must be ok");
        let result = parse(tokens.as_slice(), data_source)
            .expect("Parse result must be ok");
        let condition_tokens = Scanner::scan("                          f.foo_id = b.bar_id")
            .expect("Scanner result must be ok");
        let condition = parse(condition_tokens.as_slice(), expression)
            .expect("Parse result must be ok");
        let (left, right) = extract_join(result, JoinType::Cross, Some(JoinCondition::Expression(condition)));
        assert_table(left, "foo", Some("f"));
        assert_table(right, "bar", Some("b"));
    }
}
