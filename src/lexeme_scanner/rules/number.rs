//! Правило "Число"
use helpers::parse_number_literal;
use super::*;

/// Функция-тест, проверяющая начало ввода на предмет начала числа
#[inline]
pub fn is_number_begin(input: &[u8]) -> bool {
    let len = input.len();
    if len == 0 {
        return false;
    }
    match input[0] as char {
        '-' | '.' => (len > 1) && (input[1] as char).is_digit(10),
        c => c.is_digit(10),
    }
}

/**
    Правило "Число".

    Обрабатывает встреченное в начале ввода число.
    Учитывает знак минус перед числом (тогда оно станет отрицательным),
    двоичные числа, начинающиеся с `0b`,
    восьмеричные числа, начинающиеся с `0` и `0o`,
    обычные десятеничные числа,
    шеснадцатеричные числа, начинающиеся с `0x`,
    дробыне числа, содержащие в теле точку (`.`).

    <b>Важно:</b>
    Число не может состоять из одного префикса (`0b`,  `0`, `0o`, `0x`).
    После него обязана находиться цифра или десятичная точка,
    после которой так же обязана стоять цифра, иначе точка не будет считаться частью числа.

    Возвращает ошибку `MustBeGot` в случае, если начало ввода не является цифрой или знаком `-`.
*/
#[inline]
pub fn number(input: &[u8]) -> BatcherResult {
    parse_number_literal(input)
        .map(|(kind, _, size)| (kind, size))
}
