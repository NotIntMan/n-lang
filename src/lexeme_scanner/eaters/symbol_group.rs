//! Поедатель символов

use super::*;

use self::basics::*;

array!(const SYMBOLS_1: char =
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
    '.',
    ',',
    '(',
    ')',
    '{',
    '}',
    '[',
    ']',
    '!',
    '*',
);

array!(const SYMBOLS_2: [char; 2] =
    ['|', '|'],
    ['&', '&'],
    ['<', '<'],
    ['>', '>'],
    ['>', '='],
    ['<', '='],
    ['.', '.'],
);

/// Функция-тест, проверяющая является ли символ началом группы символов или самостоятельным специальным символом
#[inline]
pub fn is_begin_of_group(c: char) -> bool {
    if SYMBOLS_1.contains(&c) {
        return true
    }
    for arr in SYMBOLS_2.iter() {
        if arr[0] == c { return true }
    }
    false
}

/**
    Поедатель символов

    При встрече со специальным символом или их группой, поглощает их и возвращает `TokenKind::SymbolGroup`
*/
pub fn eat_symbol_group(it: &mut ScannerCursor) -> BatcherResult {
    assert_peek_pred(it, is_begin_of_group, "a specific symbol")?;
    let a = match it.next() {
        Some(c) => c,
        None => unreachable!(),
    };
    let b = match it.peek() {
        Some(c) => c,
        None => return Ok(TokenKind::SymbolGroup),
    };
    if SYMBOLS_2.contains(&[a, b]) {
        it.next();
    }
    Ok(TokenKind::SymbolGroup)
}
