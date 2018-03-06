/*!
    Модуль "Прочие запросы к БД".

    # Грамматика

    ```md
    updating_value = "default" | expression

    updating_assignment = property_path "=" updating_value

    limit_clause = "limit" u32_literal

    inserting_priority = ["low_priority" | "delayed" | "high_priority"]

    value_list = "(" comma_list(expression) ")"

    property_list = "(" comma_list(property_path) ")"

    inserting_source =
        | [property_list] ("value" | "values") comma_list(value_list)
        | "set" comma_list(updating_assignment)
        | [property_list] selection

    inserting_on_duplicate_key_update = "on" "duplicate" "key" "update" comma_list(updating_assignment)

    where_clause = select_condition("where")

    order_by_clause = select_sorting("order")

    updating =
        "update" ["low_priority"] ["ignore"] data_source
        "set" comma_list(updating_assignment)
        [where_clause] [order_by_clause] [limit_clause]

    inserting =
        "insert" inserting_priority ["ignore"]
        "into" data_source inserting_source
        [inserting_on_duplicate_key_update]

    deleting =
        "delete" ["low_priority"] ["quick"] ["ignore"]
        "from" data_source [where_clause] [order_by_clause] [limit_clause]
    ```


    Правила `comma_list` и `u32_literal` определены в модуле `parser_basics`.

    Правило `data_source` определено в модуле `syntax_parser::data_sources`.

    Правило `expression` определено в модуле `syntax_parser::expressions`.

    Правило `property_path` определено в модуле `syntax_parser::others`.

    Правила `selection`, `select_condition` и `select_sorting` определены в модуле `syntax_parser::selections`.
*/

pub mod definitions;
pub mod rules;

pub use self::definitions::*;
pub use self::rules::*;
