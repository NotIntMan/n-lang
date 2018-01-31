//! Модуль, содержащий функции-поедатели ввода

use super::*;

pub mod basics;
pub mod string;
pub mod number;
pub mod symbol_group;
pub mod whitespace;
pub mod word;

/// Композитор всех прочих "поедателей", расположенных в этом модуле
pub fn scan(it: &mut ScannerCursor) -> BatcherResult {
    let peek = match it.peek() {
        Some(p) => p,
        None => return Ok(TokenKind::EndOfInput),
    };

    if whitespace::is_whitespace(peek) {
        whitespace::eat_whitespace(it)
    } else if peek == '"' {
        string::eat_string(it, '"', TokenKind::StringLiteral)
    } else if peek == '\'' {
        string::eat_string(it, '\'', TokenKind::BracedExpressionLiteral)
    } else if number::is_number_begin(it) {
        number::eat_number(it)
    } else if word::is_letter(peek) {
        word::eat_word(it)
    } else if symbol_group::is_begin_of_group(peek) {
        symbol_group::eat_symbol_group(it)
    } else {
        unimplemented!()
    }
}
