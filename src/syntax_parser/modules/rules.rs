use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    item_position,
    keyword,
    none,
    symbols,
    symbol_position,
    ParserResult,
};
use syntax_parser::compound_types::{
    attributes,
    compound_type,
    struct_body,
};
use syntax_parser::others::module_path;
use syntax_parser::functions::function_definition;
use super::*;

parser_rule!(data_type_definition(i) -> ModuleDefinitionValueAST<'source> {
    do_parse!(i,
        apply!(keyword, "struct") >>
        name: identifier >>
        body: compound_type >>
        (ModuleDefinitionValueAST::DataType(DataTypeDefinitionAST { name, body }))
    )
});

parser_rule!(table_definition(i) -> ModuleDefinitionValueAST<'source> {
    do_parse!(i,
        begin: symbol_position >>
        apply!(keyword, "table") >>
        name: identifier >>
        body: struct_body >>
        pos: apply!(item_position, begin) >>
        (ModuleDefinitionValueAST::Table(TableDefinitionAST { name, pos, body }))
    )
});

parser_rule!(function_definition_in_module(i) -> ModuleDefinitionValueAST<'source> {
    do_parse!(i,
        def: function_definition >>
        (ModuleDefinitionValueAST::Function(def))
    )
});

parser_rule!(module_definitions(i) -> ModuleDefinitionValueAST<'source> {
    do_parse!(i,
        apply!(keyword, "mod") >>
        name: identifier >>
        apply!(symbols, "{") >>
        items: module >>
        apply!(symbols, "}") >>
        (ModuleDefinitionValueAST::Module(ModuleDefinitionAST { name, items }))
    )
});


parser_rule!(external_item_definition(i) -> ModuleDefinitionValueAST<'source> {
    do_parse!(i,
        apply!(keyword, "use") >>
        path: module_path >>
        tail: alt!(
            do_parse!(
                apply!(symbols, "::") >>
                apply!(symbols, "*") >>
                (ExternalItemTailAST::Asterisk)
            )
            | do_parse!(
                apply!(keyword, "as") >>
                alias: identifier >>
                (ExternalItemTailAST::Alias(alias))
            )
            | none => { |_| ExternalItemTailAST::None }
        ) >>
        apply!(symbols, ";") >>
        (ModuleDefinitionValueAST::Import(ExternalItemImportAST { path, tail }))
    )
});

parser_rule!(module_definition_item(i) -> ModuleDefinitionItemAST<'source> {
    do_parse!(i,
        attributes: attributes >>
        public: opt!(apply!(keyword, "pub")) >>
        value: alt!(
            data_type_definition
            | table_definition
            | function_definition_in_module
            | module_definitions
            | external_item_definition
        ) >>
        (ModuleDefinitionItemAST {
            public: public.is_some(),
            attributes,
            value,
        })
    )
});

/// Выполняет разбор грамматики модуля
pub fn module<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Vec<ModuleDefinitionItemAST<'source>>> {
    many0!(input, module_definition_item)
}
