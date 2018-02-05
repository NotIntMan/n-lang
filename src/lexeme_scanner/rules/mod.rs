//! Модуль, содержащий функции-правила для обработки ввода

use super::*;

pub mod basics;
pub mod string;
pub mod number;
pub mod symbol_group;
pub mod whitespace;
pub mod word;

/// Композитор всех прочих "правил", расположенных в этом модуле
pub fn scan(input: &[u8]) -> BatcherResult {
    if input.len() == 0 {
        #[cfg(test)] trace!("Scanner found end of the input");
        return Ok((TokenKind::EndOfInput, 0));
    }
    let peek = input[0] as char;
    if whitespace::is_whitespace(peek) {
        #[cfg(test)] trace!("Scanner found a whitespace");
        whitespace::whitespace(input)
    } else if peek == '"' {
        #[cfg(test)] trace!("Scanner found a string literal");
        string::string(input, '"', TokenKindLess::StringLiteral)
    } else if peek == '\'' {
        #[cfg(test)] trace!("Scanner found a braced literal");
        string::string(input, '\'', TokenKindLess::BracedExpressionLiteral)
    } else if number::is_number_begin(input) {
        #[cfg(test)] trace!("Scanner found a number literal");
        number::number(input)
    } else if word::is_letter(peek) {
        #[cfg(test)] trace!("Scanner found a word");
        word::word(input)
    } else if symbol_group::is_begin_of_group(peek) {
        #[cfg(test)] trace!("Scanner found a group of symbols");
        symbol_group::symbol_group(input)
    } else {
        #[cfg(test)] trace!("Scanner found something strange");
        Err((ScannerErrorKind::UnexpectedInput(peek), 1))
    }
}
