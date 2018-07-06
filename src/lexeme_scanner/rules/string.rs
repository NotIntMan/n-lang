//! Правило "Строка"

use self::basics::*;
use super::*;

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
    let mut length = 0;
    loop {
        match extract_char(input, result, "string body's symbol")? {
            '\\' => {
                extract_char(input, result + 1, "escape symbol")?;
                result += 2;
                length += 1;
            }
            c => {
                result += 1;
                if c == bracket {
                    return Ok((TokenKind::new_string_literal(kind, length), result));
                }
                length += 1;
            }
        }
    }
}
