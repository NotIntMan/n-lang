use lexeme_scanner::Token;
use parser_basics::{
    identifier,
    keyword,
    symbols,
    ParserResult,
};
use desc_lang::compounds::{
    attributes,
    compound_type,
    struct_body,
};
use man_lang::others::module_path;
use desc_lang::functions::function_definition;
use super::*;

parser_rule!(data_type_definition(i) -> ModuleDefinitionValue<'source> {
    do_parse!(i,
        apply!(keyword, "struct") >>
        name: identifier >>
        body: compound_type >>
        (ModuleDefinitionValue::DataType(DataTypeDefinition { name, body }))
    )
});

parser_rule!(table_definition(i) -> ModuleDefinitionValue<'source> {
    do_parse!(i,
        apply!(keyword, "table") >>
        name: identifier >>
        body: struct_body >>
        (ModuleDefinitionValue::Table(TableDefinition { name, body }))
    )
});

parser_rule!(function_definition_in_module(i) -> ModuleDefinitionValue<'source> {
    do_parse!(i,
        def: function_definition >>
        (ModuleDefinitionValue::Function(def))
    )
});

parser_rule!(module_definitions(i) -> ModuleDefinitionValue<'source> {
    do_parse!(i,
        apply!(keyword, "mod") >>
        name: identifier >>
        apply!(symbols, "{") >>
        items: module >>
        apply!(symbols, "}") >>
        (ModuleDefinitionValue::Module(ModuleDefinition { name, items }))
    )
});

parser_rule!(external_item_definition(i) -> ModuleDefinitionValue<'source> {
    do_parse!(i,
        apply!(keyword, "use") >>
        path: module_path >>
        alias: opt!(do_parse!(
            apply!(keyword, "as") >>
            name: identifier >>
            (name)
        )) >>
        opt!(apply!(symbols, ";")) >>
        (ModuleDefinitionValue::Import(ExternalItemImport { path, alias }))
    )
});

parser_rule!(module_definition_item(i) -> ModuleDefinitionItem<'source> {
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
        (ModuleDefinitionItem {
            public: public.is_some(),
            attributes,
            value,
        })
    )
});

/// Выполняет разбор грамматики модуля
pub fn module<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Vec<ModuleDefinitionItem<'source>>> {
    many0!(input, module_definition_item)
}
