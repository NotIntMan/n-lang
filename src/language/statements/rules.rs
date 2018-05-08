use helpers::extract;
use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    keyword,
    list,
    symbols,
    ParserResult,
};
use language::{
    data_type,
    deleting,
    expression,
    inserting,
    selection,
    updating,
};
use super::*;

parser_rule!(stmt_source(i) -> StatementSourceAST<'source> {
    alt!(i,
        selection => { |x| StatementSourceAST::Selection(x) }
        | expression => { |x| StatementSourceAST::Expression(x) }
    )
});

parser_rule!(variable_definition(i) -> StatementAST<'source> {
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
        (StatementAST::VariableDefinition {
            name,
            data_type,
            default_value,
        })
    )
});

parser_rule!(variable_assignment(i) -> StatementAST<'source> {
    do_parse!(i,
        name: identifier >>
        apply!(symbols, ":=") >>
        source: stmt_source >>
        (StatementAST::VariableAssignment {
            name,
            source,
        })
    )
});

parser_rule!(condition(i) -> StatementAST<'source> {
    do_parse!(i,
        apply!(keyword, "if") >>
        condition: expression >>
        then_body: map!(block, |stmt| Box::new(stmt)) >>
        else_body: opt!(do_parse!(
            apply!(keyword, "else") >>
            stmt: block >>
            (Box::new(stmt))
        )) >>
        (StatementAST::Condition {
            condition,
            then_body,
            else_body,
        })
    )
});

parser_rule!(simple_cycle(i) -> StatementAST<'source> {
    do_parse!(i,
        apply!(keyword, "loop") >>
        body: map!(block, |stmt| Box::new(stmt)) >>
        (StatementAST::Cycle {
            cycle_type: CycleType::Simple,
            body,
        })
    )
});

parser_rule!(pre_predicated_cycle(i) -> StatementAST<'source> {
    do_parse!(i,
        apply!(keyword, "while") >>
        predicate: expression >>
        body: map!(block, |stmt| Box::new(stmt)) >>
        (StatementAST::Cycle {
            cycle_type: CycleType::PrePredicated(predicate),
            body,
        })
    )
});

parser_rule!(post_predicated_cycle(i) -> StatementAST<'source> {
    do_parse!(i,
        apply!(keyword, "do") >>
        body: map!(block, |stmt| Box::new(stmt)) >>
        apply!(keyword, "while") >>
        predicate: expression >>
        (StatementAST::Cycle {
            cycle_type: CycleType::PostPredicated(predicate),
            body,
        })
    )
});

parser_rule!(cycle_control(i) -> StatementAST<'source> {
    do_parse!(i,
        operator: alt!(
            apply!(keyword, "break") => { |_| CycleControlOperator::Break }
            | apply!(keyword, "continue") => { |_| CycleControlOperator::Continue }
        ) >>
        name: opt!(identifier) >>
        (StatementAST::CycleControl {
            operator,
            name,
        })
    )
});

parser_rule!(return_stmt(i) -> StatementAST<'source> {
    do_parse!(i,
        apply!(keyword, "return") >>
        value: opt!(stmt_source) >>
        (StatementAST::Return {
            value,
        })
    )
});

parser_rule!(pub block(i) -> StatementAST<'source> {
    do_parse!(i,
        apply!(symbols, "{") >>
        statements: apply!(list, statement, prepare!(symbols(";"))) >>
        apply!(symbols, "}") >>
        (match statements.len() {
            0 => StatementAST::Nothing,
            1 => {
                let mut statements = statements;
                extract(&mut statements[0])
            },
            _ => StatementAST::Block { statements },
        })
    )
});

parser_rule!(expr(i) -> StatementAST<'source> {
    do_parse!(i,
        expression: expression >>
        (StatementAST::Expression{
            expression,
        })
    )
});

parser_rule!(request(i) -> StatementAST<'source> {
    alt!(i,
        deleting => { |request| StatementAST::DeletingRequest { request } }
        | inserting => { |request| StatementAST::InsertingRequest { request } }
        | updating => { |request| StatementAST::UpdatingRequest { request } }
    )
});

/// Выполняет разбор императивных высказываний
pub fn statement<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, StatementAST<'source>> {
    alt!(input,
        request
        | variable_definition
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
