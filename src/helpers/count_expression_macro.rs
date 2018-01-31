//! Макрос-помощник. Подсчитывает количество переданных ему аргументов.

/**
    Макрос-помощник. Подсчитывает количество переданных ему аргументов.

    # Примеры

    ```rust
    # #[macro_use]
    # extern crate n_transpiler;
    # fn main() {
    assert_eq!(count_expressions!(), 0);
    assert_eq!(count_expressions!(a), 1);
    assert_eq!(count_expressions!(b, 2), 2);
    assert_eq!(count_expressions!(a, b, 2), 3);
    assert_eq!(count_expressions!(a, b, x, 2), 4);
    # }
    ```

    # Пояснение

    Макрос реализован рекурсивно. Т.е. математически он может быть определён при помощи следующей системы:

    * `count_expressions!() => 0`

    * `count_expressions!(a) => 1`

    * `count_expressions!(a, ...b) => 1 + count_expressions!(...b)`, где `длина b > 0`
*/
#[macro_export]
macro_rules! count_expressions {
    () => { 0 };
    ( $f: expr ) => { 1 };
    ( $f: expr, $( $e: expr ),+ ) => {
        1 + count_expressions!($($e),+)
    };
}
