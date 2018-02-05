//! Поедатель пробелов

use super::*;

use self::basics::*;

/// Функция тест, проверяющая является ли символ пробельным
#[inline]
pub fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

//#[inline]
//fn option_is_whitespace(o: Option<char>) -> bool {
//    if let Some(c) = o {
//        c.is_whitespace()
//    } else {
//        false
//    }
//}
//
///// Поедатель пробелов. Поглощает пробел или их группу. Переносы строки, возвраты каретки так же поглощаются.
//pub fn eat_whitespace(it: &mut ScannerCursor) -> BatcherResult {
//    assert_peek_pred(it, is_whitespace, "whitespace")?;
//    it.next();
//    while option_is_whitespace(it.peek()) {
//        it.next();
//    }
//    Ok(TokenKind::Whitespace)
//}

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
