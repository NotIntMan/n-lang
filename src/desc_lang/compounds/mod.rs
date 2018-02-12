/*!
    Модуль "Составные типы данных".

    # Грамматика

    ```
    attribute = "#[" identifier [(...identifier)] "]"

    attributes = ...attribute

    struct_field = attributes identifier ":" data_type

    tuple_field = attributes data_type

    struct_body = attributes "{" ...struct_field "}"

    tuple_body = attributes "(" ...tuple_field ")"

    data_type = struct_body | tuple_body | primitive_data_type | identifier
    ```

    Правило `identifier` определено в модуле `parser_basics`.
    Правило `primitive_data_type` определено в модуле `desc_lang::primitives`.
*/

pub mod definitions;
pub mod rules;

pub use self::definitions::*;
pub use self::rules::*;
