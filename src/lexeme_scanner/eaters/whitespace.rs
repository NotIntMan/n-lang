//! Поедатель пробелов

use super::*;

use self::basics::*;

/// Функция тест, проверяющая является ли символ пробельным
#[inline]
pub fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
}

#[inline]
fn option_is_whitespace(o: Option<char>) -> bool {
    if let Some(c) = o {
        c.is_whitespace()
    } else {
        false
    }
}

/// Поедатель пробелов. Поглощает пробел или их группу. Переносы строки, возвраты каретки так же поглощаются.
pub fn eat_whitespace(it: &mut ScannerCursor) -> BatcherResult {
    assert_peek_pred(it, is_whitespace, "whitespace")?;
    it.next();
    while option_is_whitespace(it.peek()) {
        it.next();
    }
    Ok(TokenKind::Whitespace)
}
