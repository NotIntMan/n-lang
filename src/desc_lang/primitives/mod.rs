/*!
    Модуль "Примитивные типы данных".

    # Грамматика

    ```
    single_size = [(u32)]

    bit = "bit" single_size

    unsigned_zerofill = ["unsigned"] ["zerofill"]

    integer = unsigned_zerofill ["tiny" | "small" | "medium" | "big"] "integer"

    decimal = unsigned_zerofill "decimal" [(u32[, u32])]

    float_size = [(u32, u32)]

    float = unsigned_zerofill "float" float_size

    double = unsigned_zerofill "double" float_size

    number_type = bit | "boolean" | integer | decimal | float | double

    datetime_type = "date" | "time" single_size | "datetime" single_size | "timestamp" single_size

    year_type = "year4" | "year2" | "year"

    character_set_type = "binary" | "utf8"

    character_set = "character" "set" character_set_type

    string_type = "varchar" single_size [character_set] | "text" [character_set]

    pub primitive_data_type = number_type | datetime_type | year_type | string_type
    ```
*/

pub mod definitions;
pub mod rules;

pub use self::definitions::*;
pub use self::rules::*;
