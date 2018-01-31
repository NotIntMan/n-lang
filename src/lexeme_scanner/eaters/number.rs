//! Поедатель чисел
use super::*;

use self::basics::*;

/// Функция-тест, проверяющая пик курсора на предмет начала числа
#[inline]
pub fn is_number_begin(it: &mut ScannerCursor) -> bool {
    let c = match it.get(0) {
        Some(c) => *c,
        None => return false,
    };
    if c.is_digit(10) {
        return true;
    }
    if (c == '-') || (c == '.') {
        if let Some(d) = it.get(1) {
            return d.is_digit(10);
        }
    }
    false
}

/// Внутреняя функция-помощник, помогающая определить есть ли число после обнаруженной точки
#[inline]
fn is_digit_after_point(it: &mut ScannerCursor) -> bool {
    if  it.peek() == Some('.') {
        if let Some(c) = it.get(1) {
            if c.is_digit(16) {
                return true;
            }
        }
    }
    false
}

/**
    Поедатель чисел

    Поглощает встреченные числа.
    Учитывает знак минус перед числом (тогда оно станет отрицательным),
    двоичные числе, начинающиеся с `0b`,
    восьмеричные числа, начинающиеся с `0` и `0o`,
    обычные десятеничные числа,
    шеснадцатеричные числа, начинающиеся с `0x`,
    дробыне числа, содержащие в теле точку (`.`).

    <b>Важно:</b>
    Число не может состоять из одного префикса (`0b`,  `0`, `0o`, `0x`).
    После него обязана находиться цифра или десятичная точка.
*/
pub fn eat_number(it: &mut ScannerCursor) -> BatcherResult {
    let negative = if it.peek() == Some('-') {
        it.next();
        true
    } else { false };
    let mut fractional = if it.peek() == Some('.') {
        true
    } else {
        assert_peek_pred(it, |c| c.is_digit(10), "a decimal digit")?;
        false
    };
    let f = match it.next() {
        Some(c) => c,
        None => return Err(ScannerErrorKind::UnexpectedEnd),
    };
    let (radix, need_to_check) = if f == '0' {
        match it.peek() {
            Some('x') => {
                it.next();
                (16, true)
            }
            Some('o') => {
                it.next();
                (8, true)
            }
            Some('b') => {
                it.next();
                (2, true)
            }
            Some('.') => {
                (10, false)
            }
            Some(_) => (8, false),
            None => return Ok(TokenKind::NumberLiteral { negative, radix: 10, fractional: false }),
        }
    } else {
        (10, false)
    };
    if need_to_check {
        match it.peek() {
            Some('.') => {},
            Some(_) => assert_peek_pred(it, |c| c.is_digit(16), "a digit")?,
            None => return Err(ScannerErrorKind::UnexpectedEnd),
        }
    }
    'parse_cycle: loop {
        if let Some(c) = it.peek() {
            if c.is_digit(16) {
                if c.is_digit(radix) {
                    it.next();
                    continue 'parse_cycle;
                } else {
                    return Err(ScannerErrorKind::NotInRadix(c, radix));
                }
            }
            if c == '.' {
                if !fractional && is_digit_after_point(it) {
                    fractional = true;
                    it.next();
                    continue 'parse_cycle;
                }
                break 'parse_cycle;
            }
        }
        break 'parse_cycle;
    }
    Ok(TokenKind::NumberLiteral { negative, fractional, radix })
}
