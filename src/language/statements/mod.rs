/*!
    Модуль "Высказывания".

    # Грамматика

    ```md
    variable_definition = "let" identifier [":" data_type] [":=" expression]

    variable_assignment = identifier ":=" expression

    condition = "if" expression block ["else" block]

    simple_cycle = "loop" block

    pre_predicated_cycle = "while" expression block

    post_predicated_cycle = "do" block "while" expression

    cycle_control =
        | "break" [identifier]
        | "continue" [identifier]

    return_stmt = "return" [expression]

    block = "{" list(statement, ";") "}"

    statement =
        | variable_definition
        | variable_assignment
        | condition
        | simple_cycle
        | pre_predicated_cycle
        | post_predicated_cycle
        | cycle_control
        | return_stmt
        | block
        | expression
    ```

    Правила `list` и `identifier` определены в модуле `parser_basics`.

    Правило `expression` определено в модуле `language::expressions`.
*/

pub use self::definitions::*;
pub use self::rules::*;

pub mod definitions;
pub mod rules;

