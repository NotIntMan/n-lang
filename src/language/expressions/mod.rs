/*!
    Модуль "Выражения".

    # Грамматика

    ```md
    literal =
        | token(NumberLiteral)
        | token(StringLiteral)
        | token(BracedExpressionLiteral)
        | "true" | "false" | "null"

    prefix_unary_operator =
        | "!"
        | "all"
        | "any"
        | "+"
        | "-"
        | "~"
        | "binary"
        | "row"
        | "exists"

    postfix_unary_operator =
        | "is" "null"
        | "is" "not" "null"
        | "is" "true"
        | "is" "not" "true"
        | "is" "false"
        | "is" "not" "false"
        | "is" "unknown"
        | "is" "not" "unknown"

    binary_operator =
        | "or" | "||"
        | "xor" | "^^"
        | "and" | "&&"
        | "|"
        | "^"
        | "&"
        | "<<"
        | ">>"
        | "is" "in"
        | "="
        | ">="
        | ">"
        | "<="
        | "<"
        | "like"
        | "sounds" "like"
        | "regexp"
        | "+"
        | "-"
        | "*"
        | "/"
        | "mod" | "%"
        | "div"
        | "**"
        | ".."
        
    expression =
        | literal
        | expression "." property_path
        | expression "(" comma_list(expression) ")"
        | expression module_path "(" comma_list(expression) ")"
        | prefix_unary_operator expression
        | expression postfix_unary_operator
        | expression binary_operator expression
    ```

    Правило `expression` определено в модуле `language::expressions`.

    Правила `property_path` и `module_path` определены в модуле `language::others`.

    Правила `comma_list` и `token` определены в модуле `parser_basics`.
*/

pub mod definitions;
pub mod rules;

pub use self::definitions::*;
pub use self::rules::*;

