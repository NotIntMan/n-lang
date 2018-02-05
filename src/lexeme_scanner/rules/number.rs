//! Правило "Число"
use super::*;

use self::basics::*;

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
pub fn number(input: &[u8]) -> BatcherResult {
    let len = input.len();
    if !is_number_begin(input) {
        let got = extract_char(input, 0, "begin of a number")?;
        return Err((ScannerErrorKind::must_be_got("begin of a number", got), 0));
    }
    let (
        negative,
        mut result,
    ) = if (len > 0) && ((input[0] as char) == '-') {
        (true, 1)
    } else {
        (false, 0)
    };
    let mut fractional = false;
    let radix = if extract_char(input, result, "a number")? == '0' {
        result += 1;
        if result >= len {
            return Ok((TokenKind::NumberLiteral { negative, radix: 10, fractional: false }, result));
        }
        match input[result] as char {
            'x' => { result += 1; 16 },
            'o' => { result += 1; 8 },
            'b' => { result += 1; 2 },
            '.' => {
                assert_pred(input, result + 1,
                            |c| c.is_digit(10), "a decimal digit")?;
                result += 1;
                fractional = true;
                10
            },
            _ => 8,
        }
    } else { 10 };
    'parse_cycle: loop {
        if result >= len {
            break 'parse_cycle;
        }
        let c = input[result] as char;
        if c.is_digit(16) {
            if c.is_digit(radix) {
                result += 1;
                continue 'parse_cycle;
            } else {
                return Err((ScannerErrorKind::NotInRadix(c, radix), result));
            }
        }
        if c == '.' {
            if !fractional && ((result + 1) < len) && (input[result + 1] as char).is_digit(16) {
                fractional = true;
                result += 1;
                continue 'parse_cycle;
            }
        }
        break 'parse_cycle;
    }
    Ok((TokenKind::NumberLiteral { negative, fractional, radix }, result))
}
