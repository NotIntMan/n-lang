use helpers::extract::extract;
use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    keyword,
    list,
    symbols,
    ParserResult,
};
use syntax_parser::compound_types::data_type;
use syntax_parser::expressions::expression;
use syntax_parser::selections::selection;
use super::*;

parser_rule!(stmt_source(i) -> StatementSource<'source> {
    alt!(i,
        selection => { |x| StatementSource::Selection(x) }
        | expression => { |x| StatementSource::Expression(x) }
    )
});

parser_rule!(variable_definition(i) -> Statement<'source> {
    do_parse!(i,
        apply!(keyword, "let") >>
        name: identifier >>
        data_type: opt!(do_parse!(
            apply!(symbols, ":") >>
            data_type: data_type >>
            (data_type)
        )) >>
        default_value: opt!(do_parse!(
            apply!(symbols, ":=") >>
            source: stmt_source >>
            (source)
        )) >>
        (Statement::VariableDefinition {
            name,
            data_type,
            default_value,
        })
    )
});

parser_rule!(variable_assignment(i) -> Statement<'source> {
    do_parse!(i,
        name: identifier >>
        apply!(symbols, ":=") >>
        source: stmt_source >>
        (Statement::VariableAssignment {
            name,
            source,
        })
    )
});

parser_rule!(condition(i) -> Statement<'source> {
    do_parse!(i,
        apply!(keyword, "if") >>
        condition: expression >>
        then_body: map!(block, |stmt| Box::new(stmt)) >>
        else_body: opt!(do_parse!(
            apply!(keyword, "else") >>
            stmt: block >>
            (Box::new(stmt))
        )) >>
        (Statement::Condition {
            condition,
            then_body,
            else_body,
        })
    )
});

parser_rule!(simple_cycle(i) -> Statement<'source> {
    do_parse!(i,
        apply!(keyword, "loop") >>
        body: map!(block, |stmt| Box::new(stmt)) >>
        (Statement::Cycle {
            cycle_type: CycleType::Simple,
            body,
        })
    )
});

parser_rule!(pre_predicated_cycle(i) -> Statement<'source> {
    do_parse!(i,
        apply!(keyword, "while") >>
        predicate: expression >>
        body: map!(block, |stmt| Box::new(stmt)) >>
        (Statement::Cycle {
            cycle_type: CycleType::PrePredicated(predicate),
            body,
        })
    )
});

parser_rule!(post_predicated_cycle(i) -> Statement<'source> {
    do_parse!(i,
        apply!(keyword, "do") >>
        body: map!(block, |stmt| Box::new(stmt)) >>
        apply!(keyword, "while") >>
        predicate: expression >>
        (Statement::Cycle {
            cycle_type: CycleType::PostPredicated(predicate),
            body,
        })
    )
});

parser_rule!(cycle_control(i) -> Statement<'source> {
    do_parse!(i,
        operator: alt!(
            apply!(keyword, "break") => { |_| CycleControlOperator::Break }
            | apply!(keyword, "continue") => { |_| CycleControlOperator::Continue }
        ) >>
        name: opt!(identifier) >>
        (Statement::CycleControl {
            operator,
            name,
        })
    )
});

parser_rule!(return_stmt(i) -> Statement<'source> {
    do_parse!(i,
        apply!(keyword, "return") >>
        value: opt!(stmt_source) >>
        (Statement::Return {
            value,
        })
    )
});

parser_rule!(pub block(i) -> Statement<'source> {
    do_parse!(i,
        apply!(symbols, "{") >>
        statements: apply!(list, statement, prepare!(symbols(";"))) >>
        apply!(symbols, "}") >>
        (match statements.len() {
            0 => Statement::Nothing,
            1 => {
                let mut statements = statements;
                extract(&mut statements[0])
            },
            _ => Statement::Block { statements },
        })
    )
});

parser_rule!(expr(i) -> Statement<'source> {
    do_parse!(i,
        expression: expression >>
        (Statement::Expression{
            expression,
        })
    )
});

/// Выполняет разбор императивных высказываний
pub fn statement<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Statement<'source>> {
    alt!(input,
        variable_definition
        | variable_assignment
        | condition
        | simple_cycle
        | pre_predicated_cycle
        | post_predicated_cycle
        | cycle_control
        | return_stmt
        | block
        | expr
    )
}
