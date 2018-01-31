//! Модуль, содержащий функции-помощники. Его отличие от глобального в том, что он пригождается только в контексте "поедателей".

use super::*;

/// Функция, проверяющая пик данного итератора на предмет эквиватентности переданному символу.
/// В случае не совпадения, возвращает ошибку `ScannerError::ExpectedGot`.
#[inline]
pub fn assert_peek_eq(it: &mut ScannerCursor, item: char) -> Result<(), ScannerErrorKind> {
    if it.peek() == Some(item) {
        it.next();
        Ok(())
    } else {
        Err(ScannerErrorKind::ExpectedGot(item, it.peek()))
    }
}

/// Функция, проверяющая пик данного итератора на предмет удовлетворения условиям переданного теста.
/// В случае не удовлетворения условиям теста, возвращает ошибку `ScannerError::MustBeGot`.
#[inline]
pub fn assert_peek_pred<F: Fn(char) -> bool>(it: &mut ScannerCursor, test: F, msg: &str) -> Result<(), ScannerErrorKind> {
    if let Some(c) = it.peek() {
        if test(c) {
            return Ok(())
        }
    }
    Err(ScannerErrorKind::MustBeGot(String::from(msg), it.peek()))
}
