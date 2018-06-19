/*!
    Модуль "Источники данных".

    # Грамматика

    ```md
    table = module_path not_keyword_identifier

    join_source =
        | table
        | "(" data_source ")"
        | "(" selection ")" "as" not_keyword_identifier

    join_condition =
        | "on" expression
        | "using" "(" comma_list(property_path) ")"

    join_tail =
        | "natural" "left" ["outer"] "join" join_source  
        | "left" ["outer"] "join" join_source [join_condition]  
        | "natural" "right" ["outer"] "join" join_source  
        | "right" ["outer"] "join" join_source [join_condition]  
        | "natural" "full" ["outer"] "join" join_source  
        | "full" ["outer"] "join" join_source [join_condition]
        | "inner" "join" join_source [join_condition]
        | ["cross"] "join" join_source
        | "," join_source
    
    data_source = join_source join_tail*
    ```

    Правила `property_path` и `module_path` определены в модуле `language::others`.

    Правила `comma_list`, `keyword`, `not_keyword_identifier` и `symbols` определены в модуле `parser_basics`.

    Правило `expression` определено в модуле `language::expressions`.

    Правило `selection` определено в модуле `language::selections`.
*/

pub mod definitions;
pub mod rules;

pub use self::definitions::*;
pub use self::rules::*;
