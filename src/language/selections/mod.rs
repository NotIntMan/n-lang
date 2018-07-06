/*!
    Модуль "Выборка из БД".

    # Грамматика

    ```md
    select_distincty = ["all" | "distinct" | "distinctrow"]

    select_result_size = ["sql_small_result" | "sql_big_result" | "sql_buffer_result"]

    select_cache = ["sql_cache" | "sql_no_cache"]

    select_expression = expression ["as" identifier]

    select_result =
        | "*"
        | comma_list(select_expression)

    select_condition(W) = W expression

    select_sorting_order = ["asc" | "desc"]

    select_sorting_item = expression select_sorting_order

    select_sorting(W) = W "by" comma_list(select_sorting_item)

    select_group_by_clause = select_sorting("group") ["with" "rollup"]

    selection_limit =
        | "limit" u32_literal
        | "limit" u32_literal, u32_literal
        | "limit" u32_literal "offset" u32_literal

    selection =
        "select" select_distincty ["high_priority"] ["straight_join"]
        select_result_size select_cache select_result
        "from" data_source
        [select_condition("where")]
        [select_group_by_clause]
        [select_condition("having")]
        [select_sorting("order")]
        [selection_limit]
    ```

    Правила `comma_list` и `u32_literal` определены в модуле `parser_basics`.

    Правило `expression` определено в модуле `language::expressions`.

    Правило `data_source` определено в модуле `language::data_sources`.
*/

pub use self::definitions::*;
pub use self::rules::*;

pub mod definitions;
pub mod rules;

