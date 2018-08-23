//! Правило "Группа символов"

use self::basics::*;
use super::*;

/// Массив одиночных спец. символов
array!(pub const SYMBOLS_1: char =
    '|',
    '&',
    '^',
    '<',
    '>',
    '=',
    '+',
    '-',
    '*',
    '/',
    '%',
    '~',
    '.',
    ',',
    '(',
    ')',
    '{',
    '}',
    '[',
    ']',
    '!',
    '#',
    ':',
    ';',
);

/// Массив групп спецсимволов
array!(pub const SYMBOLS_2: [char; 2] =
    ['|', '|'],
    ['&', '&'],
    ['<', '<'],
    ['>', '>'],
    ['>', '='],
    ['<', '='],
    ['.', '.'],
    ['#', '['],
    ['^', '^'],
    ['*', '*'],
    [':', '='],
    [':', ':'],
    ['[', ']'],
);

/// Функция-тест, проверяющая является ли символ началом группы символов или самостоятельным специальным символом
#[inline]
pub fn is_begin_of_group(c: char) -> bool {
    if SYMBOLS_1.contains(&c) {
        return true;
    }
    for arr in SYMBOLS_2.iter() {
        if arr[0] == c { return true; }
    }
    false
}

/**
    Правило "Группа символов".

    При встрече со специальным символом или их группой, обрабатывает их и возвращает `TokenKind::SymbolGroup`
    Возвращает ошибку `MustBeGot` в случае, если начало ввода не входит в набор специальных символов.
*/
pub fn symbol_group(input: &[u8]) -> BatcherResult {
    assert_pred(input, 0, is_begin_of_group, "a special symbol")?;
    let a = input[0] as char;
    if input.len() > 1 {
        let b = input[1] as char;
        if SYMBOLS_2.contains(&[a, b]) {
            return Ok((TokenKind::SymbolGroup, 2));
        }
    }
    Ok((TokenKind::SymbolGroup, 1))
}
