//! Правило "Слово"

use super::*;

use self::basics::*;

/// Функция-тест, проверяющая является ли символ буквой, т.е. началом слова
#[inline]
pub fn is_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
}

#[inline]
fn is_identifier_symbol(c: char) -> bool {
    if c.is_digit(10) { return true; }
    match c {
        '_' => true,
        _ => false,
    }
}

/**
    Правило "Слово".

    Обрабатывает слово, состоящее из букв, цифр и символа `_`. Первым символом обязательно должна быть буква.
    Возвращает ошибку `MustBeGot` в случае, если в начале ввода не буква.
*/
pub fn word(input: &[u8]) -> BatcherResult {
    assert_pred(input, 0, is_letter, "a letter")?;
    let mut result = 1;
    let len = input.len();
    while (result < len) && (is_letter(input[result] as char) || is_identifier_symbol(input[result] as char)) {
        result += 1;
    }
    Ok((TokenKind::Word, result))
}
