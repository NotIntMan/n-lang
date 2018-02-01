//! Макрос-помощник. Определяет фиксированный массив с известным во время компиляции размером.

/**
    Макрос-помощник. Определяет фиксированный массив с известным во время компиляции размером.

    В зависимости от начала ввода, определяет массив в качестве константы, неизменяемой или изменяемой переменной.
    Константы могут становиться публичными.

    # Примеры

    Следующий код...

    ```rust
    # #[macro_use]
    # extern crate n_transpiler;
    # fn main() {
    array!(pub const U8_PUBLIC_CONST_ARRAY: usize = 9);
    array!(const USIZE_CONST_ARRAY: usize = 1, 2, 3);
    array!(let char_let_array: char = 'a', 'b', 'c', 'd');
    array!(let mut option_bool_mut_array: Option<bool> =
        Some(true),
        Some(false),
        Some(false),
        None,
        Some(true),
        None,
    );
    # }
    ```

    ...является эвивалентным следующей далее записи.

    ```rust
    pub const U8_PUBLIC_CONST_ARRAY: [usize; 1] = [9];
    const USIZE_CONST_ARRAY: [usize; 3] = [1, 2, 3];
    let char_let_array: [char; 4] = ['a', 'b', 'c', 'd'];
    let mut option_bool_mut_array: [Option<bool>; 6] = [
        Some(true),
        Some(false),
        Some(false),
        None,
        Some(true),
        None,
    ];
    ```
*/
#[macro_export]
macro_rules! array {
    (const $name: ident : $element_type: ty = $( $element: expr ),*) => {
        const $name: [$element_type; count_expressions!( $($element),* )] = [$($element),*];
    };
    (const $name: ident : $element_type: ty = $( $element: expr ),* ,) => {
        const $name: [$element_type; count_expressions!( $($element),* )] = [$($element),*];
    };
    (pub const $name: ident : $element_type: ty = $( $element: expr ),*) => {
        pub const $name: [$element_type; count_expressions!( $($element),* )] = [$($element),*];
    };
    (pub const $name: ident : $element_type: ty = $( $element: expr ),* ,) => {
        pub const $name: [$element_type; count_expressions!( $($element),* )] = [$($element),*];
    };
    (let $name: ident : $element_type: ty = $( $element: expr ),*) => {
        let $name: [$element_type; count_expressions!( $($element),* )] = [$($element),*];
    };
    (let $name: ident : $element_type: ty = $( $element: expr ),* ,) => {
        let $name: [$element_type; count_expressions!( $($element),* )] = [$($element),*];
    };
    (let mut $name: ident : $element_type: ty = $( $element: expr ),*) => {
        let mut $name: [$element_type; count_expressions!( $($element),* )] = [$($element),*];
    };
    (let mut $name: ident : $element_type: ty = $( $element: expr ),* ,) => {
        let mut $name: [$element_type; count_expressions!( $($element),* )] = [$($element),*];
    };
}
