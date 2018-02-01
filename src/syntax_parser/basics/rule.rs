use super::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
};

#[derive(Debug, Clone, PartialEq)]
pub struct RuleOption<R>(R);

impl<'a, 'b, R: LexemeParser<'a, 'b>> LexemeParser<'a, 'b> for RuleOption<R> {
    type Result = Option<R::Result>;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let begin = cursor.index;
        let result = self.0.parse(cursor);
        match result {
            Ok(v) => { Ok(Some(v)) },
            Err(_) => {
                cursor.index = begin;
                Ok(None)
            },
        }
    }
}

