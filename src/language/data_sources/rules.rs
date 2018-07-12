use language::{
    expression,
    ExpressionAST,
    module_path,
    selection,
};
use lexeme_scanner::Token;
use parser_basics::{
    keyword,
    not_keyword_identifier,
    ParserResult,
    symbols,
};
use super::*;

parser_rule!(join_condition(i) -> ExpressionAST<'source> {
    do_parse!(i,
        apply!(keyword, "on") >>
        x: expression >>
        (x)
    )
});

parser_rule!(table(i) -> DataSourceAST<'source> {
    do_parse!(i,
        name: module_path >>
        alias: opt!(not_keyword_identifier) >>
        (DataSourceAST::Table { name, alias })
    )
});

parser_rule!(join_source(i) -> DataSourceAST<'source> {
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
            (DataSourceAST::Selection { query: Box::new(query), alias })
        )
    )
});

type JoinTail<'source> = (JoinType, Option<ExpressionAST<'source>>, DataSourceAST<'source>);
parser_rule!(join_tail(i) -> JoinTail<'source> {
    alt!(i,
        do_parse!(
            apply!(keyword, "left") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            source: join_source >>
            condition: opt!(join_condition) >>
            ((JoinType::Left, condition, source))
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

fn fold_join<'source>(mut origin: DataSourceAST<'source>, tails: Vec<JoinTail<'source>>) -> DataSourceAST<'source> {
    for (join_type, condition, right) in tails {
        origin = DataSourceAST::Join {
            join_type,
            condition,
            left: Box::new(origin),
            right: Box::new(right),
        };
    }
    origin
}

/// Функция, выполняющая разбор источника данных запроса (таблиц и их объединений)
pub fn data_source<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, DataSourceAST<'source>> {
    do_parse!(input,
        origin: join_source >>
        tails: many0!(join_tail) >>
        (fold_join(origin, tails))
    )
}
