//! Поедатель строк

use super::*;
use self::basics::*;

///**
//    Поедатель строк
//
//    Начинает поглощение с символа кавычки, принятого в аргументе `bracket`.
//    Затем поглощает все прочие символы, а так же все экранированные символы (`\[любой символ]`).
//    Заканчивает поглощение тем же символом кавычки, с которого начал.
//*/
//pub fn eat_string(it: &mut ScannerCursor, bracket: char, kind: TokenKind) -> BatcherResult {
//    assert_peek_eq(it, bracket)?;
//    loop {
//        let c: char = match it.next() {
//            Some(c) => c,
//            None => return Err(ScannerErrorKind::UnexpectedEnd),
//        };
//        match c {
//            '\\' => {
//                if it.next().is_none() {
//                    return Err(ScannerErrorKind::UnexpectedEnd);
//                }
//            },
//            c => {
//                if c == bracket { return Ok(kind); }
//            },
//        }
//    }
//}

/**
    Правило "Строка".

    Начиная с указанного символа кавычки, обрабатывает строку и возвращает указанный тип лексемы.
    Обрабатывает все прочие, кроме кавычки, символы, а так же все экранированные символы (`\[любой символ]`).
    Заканчивает обработку тем же символом кавычки, с которого начал.

    Возвращает ошибку `MustBeGot` в случае, если начало ввода не эквивалентно символу кавычки.
    Возвращает ошибку `UnexpectedEnd` в случае, если ввод закончился, а строка - нет.
*/
pub fn string(input: &[u8], bracket: char, kind: TokenKindLess) -> BatcherResult {
    assert_eq(input, 0, bracket)?;
    let mut result = 1;
    loop {
        match extract_char(input, result, "string body's symbol")? {
            '\\' => {
                extract_char(input, result + 1, "escape symbol")?;
                result += 2;
            }
            c => {
                result += 1;
                if c == bracket {
                    return Ok((TokenKind::new_string_literal(kind, result - 2), result));
                }
            }
        }
    }
}
