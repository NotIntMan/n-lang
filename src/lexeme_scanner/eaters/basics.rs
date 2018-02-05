//! Модуль, содержащий функции-помощники. Его отличие от глобального в том, что он пригождается только в контексте "поедателей".

use super::*;

/**
    Функция, проверяющая позицию ввода на совпадение с данным символом.
    В случае не совпадения, возвращает ошибку `ExpectedGot`.
    В случае, если ввод не содержит достаточно элементов, возвщарает ошибку `UnexpectedEnd`.
*/
#[inline]
pub fn assert_eq(input: &[u8], position: usize, expect: char) -> Result<(), (ScannerErrorKind, usize)> {
    if input.len() <= position {
        return Err((ScannerErrorKind::unexpected_end_expected_char(expect), position));
    }
    let got = input[position] as char;
    if expect == got {
        Ok(())
    } else {
        Err((ScannerErrorKind::ExpectedGot(expect, got), position))
    }
}

/**
    Функция, проверяющая ввод на наличие требуемого элемента.
    В случае успеха, возвращает элемент.
    В случае, если ввод не содержит достаточно элементов, возвщарает ошибку `UnexpectedEnd`.
*/
#[inline]
pub fn extract_char<S>(input: &[u8], position: usize, msg: S) -> Result<char, (ScannerErrorKind, usize)>
    where S: Into<String> {
    if position < input.len() {
        Ok(input[position] as char)
    } else {
        Err((ScannerErrorKind::unexpected_end_expected(msg), position))
    }
}

/**
    Функция, проверяющая позицию ввода на предмет удовлетворения условиям переданного теста.
    В случае не удовлетворения условиям теста, возвращает ошибку `MustBeGot`.
    В случае, если ввод не содержит достаточно элементов, возвщарает ошибку `UnexpectedEnd`.
*/
#[inline]
pub fn assert_pred<F, S>(input: &[u8], position: usize, test: F, msg: S) -> Result<(), (ScannerErrorKind, usize)>
    where F: Fn(char) -> bool,
          S: Into<String> + Clone,
{
    let got = extract_char(input, position, msg.clone())?;
    if test(got) {
        Ok(())
    } else {
        Err((ScannerErrorKind::MustBeGot(msg.into(), got), position))
    }
}
