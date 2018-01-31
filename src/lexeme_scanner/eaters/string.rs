//! Поедатель строк

use super::*;
use self::basics::*;

/**
    Поедатель строк

    Начинает поглощение с символа кавычки, принятого в аргументе `bracket`.
    Затем поглощает все прочие символы, а так же все экранированные символы (`\[любой символ]`).
    Заканчивает поглощение тем же символом кавычки, с которого начал.
*/
pub fn eat_string(it: &mut ScannerCursor, bracket: char, kind: TokenKind) -> BatcherResult {
    assert_peek_eq(it, bracket)?;
    loop {
        let c: char = match it.next() {
            Some(c) => c,
            None => return Err(ScannerErrorKind::UnexpectedEnd),
        };
        match c {
            '\\' => {
                if it.next().is_none() {
                    return Err(ScannerErrorKind::UnexpectedEnd);
                }
            },
            c => {
                if c == bracket { return Ok(kind); }
            },
        }
    }
}
