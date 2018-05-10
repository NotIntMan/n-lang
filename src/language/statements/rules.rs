use helpers::extract;
use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    item_position,
    keyword,
    list,
    symbols,
    symbol_position,
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

parser_rule!(variable_definition(i) -> StatementASTBody<'source> {
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
        (StatementASTBody::VariableDefinition {
            name,
            data_type,
            default_value,
        })
    )
});

parser_rule!(variable_assignment(i) -> StatementASTBody<'source> {
    do_parse!(i,
        name: identifier >>
        apply!(symbols, ":=") >>
        source: stmt_source >>
        (StatementASTBody::VariableAssignment {
            name,
            source,
        })
    )
});

parser_rule!(condition(i) -> StatementASTBody<'source> {
    do_parse!(i,
        apply!(keyword, "if") >>
        condition: expression >>
        then_body: map!(block, |stmt| Box::new(stmt)) >>
        else_body: opt!(do_parse!(
            apply!(keyword, "else") >>
            stmt: block >>
            (Box::new(stmt))
        )) >>
        (StatementASTBody::Condition {
            condition,
            then_body,
            else_body,
        })
    )
});

parser_rule!(simple_cycle(i) -> StatementASTBody<'source> {
    do_parse!(i,
        apply!(keyword, "loop") >>
        body: map!(block, |stmt| Box::new(stmt)) >>
        (StatementASTBody::Cycle {
            cycle_type: CycleType::Simple,
            body,
        })
    )
});

parser_rule!(pre_predicated_cycle(i) -> StatementASTBody<'source> {
    do_parse!(i,
        apply!(keyword, "while") >>
        predicate: expression >>
        body: map!(block, |stmt| Box::new(stmt)) >>
        (StatementASTBody::Cycle {
            cycle_type: CycleType::PrePredicated(predicate),
            body,
        })
    )
});

parser_rule!(post_predicated_cycle(i) -> StatementASTBody<'source> {
    do_parse!(i,
        apply!(keyword, "do") >>
        body: map!(block, |stmt| Box::new(stmt)) >>
        apply!(keyword, "while") >>
        predicate: expression >>
        (StatementASTBody::Cycle {
            cycle_type: CycleType::PostPredicated(predicate),
            body,
        })
    )
});

parser_rule!(cycle_control(i) -> StatementASTBody<'source> {
    do_parse!(i,
        operator: alt!(
            apply!(keyword, "break") => { |_| CycleControlOperator::Break }
            | apply!(keyword, "continue") => { |_| CycleControlOperator::Continue }
        ) >>
        name: opt!(identifier) >>
        (StatementASTBody::CycleControl {
            operator,
            name,
        })
    )
});

parser_rule!(return_stmt(i) -> StatementASTBody<'source> {
    do_parse!(i,
        apply!(keyword, "return") >>
        value: opt!(stmt_source) >>
        (StatementASTBody::Return {
            value,
        })
    )
});

parser_rule!(pub block(i) -> StatementAST<'source> {
    do_parse!(i,
        begin: symbol_position >>
        apply!(symbols, "{") >>
        statements: apply!(list, statement, prepare!(symbols(";"))) >>
        apply!(symbols, "}") >>
        pos: apply!(item_position, begin) >>
        ({
            let body = match statements.len() {
                0 => StatementASTBody::Nothing,
                1 => {
                    let mut statements = statements;
                    extract(&mut statements[0]).body
                },
                _ => StatementASTBody::Block { statements },
            };
            StatementAST { body, pos }
        })
    )
});

parser_rule!(expr(i) -> StatementASTBody<'source> {
    do_parse!(i,
        expression: expression >>
        (StatementASTBody::Expression{
            expression,
        })
    )
});

parser_rule!(request(i) -> StatementASTBody<'source> {
    alt!(i,
        deleting => { |request| StatementASTBody::DeletingRequest { request } }
        | inserting => { |request| StatementASTBody::InsertingRequest { request } }
        | updating => { |request| StatementASTBody::UpdatingRequest { request } }
    )
});

/// Выполняет разбор императивных высказываний
pub fn statement<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, StatementAST<'source>> {
    do_parse!(input,
        begin: symbol_position >>
        body: alt!(
            request
            | variable_definition
            | variable_assignment
            | condition
            | simple_cycle
            | pre_predicated_cycle
            | post_predicated_cycle
            | cycle_control
            | return_stmt
            | block => { |x: StatementAST<'source>| x.body }
            | expr
        ) >>
        pos: apply!(item_position, begin) >>
        (StatementAST { body, pos })
    )
}
