/*!
    Модуль "Модули".

    # Грамматика

    ```md
    data_type_definition = "struct" identifier compound_type

    table_definition = "table" identifier struct_body

    function_definition_in_module = function_definition

    module_definitions = "mod" identifier "{" module "}"

    external_item_definition =
        | "use" module_path "::" "*"
        | "use" module_path "as" identifier
        | "use" module_path

    module_definition_item = attributes ["pub"] (
        | data_type_definition
        | table_definition
        | function_definition_in_module
        | module_definitions
        | external_item_definition
    )

    module = module_definition_item*
    ```

    Правила `attributes`, `compound_type, и `struct_body` определены в модуле `language::data_types`.

    Правило `module_path` определено в модуле `language::others`.

    Правило `identifier` определено в модуле `parser_basics`.

    Правило `function_definition` определено в модуле `language::functions`.
*/

pub use self::definitions::*;
pub use self::rules::*;

pub mod definitions;
pub mod rules;

