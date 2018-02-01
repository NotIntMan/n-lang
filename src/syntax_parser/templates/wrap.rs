use super::super::basics::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrap<OpenBracket, Element, CloseBracket>(pub OpenBracket, pub Element, pub CloseBracket);

impl<'a, 'b, OpenBracket, Element, CloseBracket> LexemeParser<'a, 'b> for Wrap<OpenBracket, Element, CloseBracket>
    where Element: LexemeParser<'a, 'b>,
          OpenBracket: LexemeParser<'a, 'b>,
          CloseBracket: LexemeParser<'a, 'b>,
{
    type Result = Element::Result;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        self.0.parse(cursor)?;
        let result = self.1.parse(cursor)?;
        self.2.parse(cursor)?;
        Ok(result)
    }
}
