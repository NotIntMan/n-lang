use super::super::basics::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List<Element, Delimiter>(pub Element, pub Delimiter);

impl<'a, 'b, Element, Delimiter> LexemeParser<'a, 'b> for List<Element, Delimiter>
    where Element: LexemeParser<'a, 'b>,
          Delimiter: LexemeParser<'a, 'b>,
{
    type Result = Vec<Element::Result>;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let mut result = Vec::new();
        let mut begin;
        'parse_loop: loop {
            begin = cursor.index;
            match self.0.parse(cursor) {
                Ok(r) => result.push(r),
                Err(_) => break 'parse_loop,
            }
            if let Err(_) = self.1.parse(cursor) {
                break 'parse_loop;
            }
        }
        cursor.index = begin;
        Ok(result)
    }
}
