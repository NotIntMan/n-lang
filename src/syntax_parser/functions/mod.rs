/*!
    Модуль "Функции".

    # Грамматика

    ```md
    type_of = ":" data_type

    argument = identifier type_of

    arguments = "(" comma_list(argument) ")"

    function_definition =
        | "extern" "fn" identifier arguments [type_of]
        | "fn" identifier arguments [type_of] block
    ```

    Правила `identifier` и `comma_list` определены в модуле `parser_basics`.

    Правило `block` определено в модуле `syntax_parser::statements`.

    Правило `data_type` определено в модуле `syntax_parser::compound_types`.
*/

pub mod definitions;
pub mod rules;
pub mod context;

pub use self::definitions::*;
pub use self::rules::*;
pub use self::context::*;
