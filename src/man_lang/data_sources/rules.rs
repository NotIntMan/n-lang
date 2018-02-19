use parser_basics::{
    comma_list,
    identifier,
    keyword,
    symbols,
};
use man_lang::expressions::expression;
use man_lang::others::property_path;
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

parser_rule!(join_tail(i) -> (JoinType, Option<JoinCondition<'source>>) {
    alt!(i,
        do_parse!(
            apply!(keyword, "natural") >>
            apply!(keyword, "left") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            ((JoinType::Left, Some(JoinCondition::Natural)))
        )
        | do_parse!(
            apply!(keyword, "left") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            condition: opt!(join_condition) >>
            ((JoinType::Left, condition))
        )
        | do_parse!(
            apply!(keyword, "natural") >>
            apply!(keyword, "right") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            ((JoinType::Right, Some(JoinCondition::Natural)))
        )
        | do_parse!(
            apply!(keyword, "right") >>
            opt!(apply!(keyword, "outer")) >>
            apply!(keyword, "join") >>
            condition: opt!(join_condition) >>
            ((JoinType::Right, condition))
        )
        | do_parse!(
            apply!(keyword, "inner") >>
            apply!(keyword, "join") >>
            condition: opt!(join_condition) >>
            ((JoinType::Cross, condition))
        )
        | do_parse!(
            opt!(apply!(keyword, "cross")) >>
            apply!(keyword, "join") >>
            ((JoinType::Cross, None))
        )
    )
});

parser_rule!(table(i) -> DataSource<'source> {
    do_parse!(i,
        name: identifier >>
        alias: opt!(do_parse!(
            opt!(apply!(keyword, "as")) >>
            alias: identifier >>
            (alias)
        )) >>
        (DataSource::Table { name, alias })
    )
});
