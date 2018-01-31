use helpers::iter_buffer::PerfectBufferCursor;

use lexeme_scanner::Token;
pub use super::parser_error::ParserError as LexemeParserError;

pub type LexemeCursor<'a, 'b> = PerfectBufferCursor<'a, Token<'b>>;
pub type LexemeParserResult<T> = Result<T, LexemeParserError>;

pub trait LexemeParser<'a, 'b> {
    type Result;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result>;
}
