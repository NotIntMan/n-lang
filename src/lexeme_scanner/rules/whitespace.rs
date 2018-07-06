//! Правило "Пробел".

use self::basics::*;
use super::*;

/// Функция тест, проверяющая является ли символ пробельным
#[inline]
pub fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

/**
    Правило "Пробел".

    Обрабатывает пробел или их группу. Переносы строки, возвраты каретки так же поглощаются.
    Возвращает ошибку `MustBeGot` в случае, если в начале ввода не пробельный символ.
*/
pub fn whitespace(input: &[u8]) -> BatcherResult {
    assert_pred(input, 0, is_whitespace, "whitespace")?;
    let mut result = 1;
    let len = input.len();
    while (result < len) && is_whitespace(input[result] as char) {
        result += 1;
    }
    Ok((TokenKind::Whitespace, result))
}
