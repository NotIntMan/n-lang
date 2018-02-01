//! Типаж для правил синтаксического разбора лексем, сгенерированных модулем `lexeme_scanner`

use helpers::iter_buffer::PerfectBufferCursor;

use lexeme_scanner::Token;

use super::ParserError;

pub type LexemeCursor<'a, 'b> = PerfectBufferCursor<'a, Token<'b>>;
pub type LexemeParserResult<T> = Result<T, ParserError>;

/**
    Типаж для правил синтаксического разбора лексем, сгенерированных модулем `lexeme_scanner`

    Ассоциированный тип Result отражает тип данных, возвращаемых правилов.
    Это позволяет возвращать что угодно и не нарушать систему типов разбора.
    Функция `parse` должна, подобно поедателю, "сдвинуть" позицию курсора ровно на то число лексем,
    которое необходимо для разбора реализуемого правила и, в случае неудачи, вернуть ошибку типа `ParserError`.
*/
pub trait LexemeParser<'a, 'b> {
    type Result;
    /// Выполняет попытку синтаксического разбора лексем данным правилом.
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result>;
}
