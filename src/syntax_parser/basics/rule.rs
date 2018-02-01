use helpers::num_range::NumRange;

use super::{
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
};

#[derive(Debug, Clone, PartialEq)]
pub struct RuleOption<R>(pub R);

impl<'a, 'b, R: LexemeParser<'a, 'b>> LexemeParser<'a, 'b> for RuleOption<R> {
    type Result = Option<R::Result>;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let begin = cursor.index;
        let result = self.0.parse(cursor);
        match result {
            Ok(v) => { Ok(Some(v)) }
            Err(_) => {
                cursor.index = begin;
                Ok(None)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBranch<A, B>(pub A, pub B);

impl<'a, 'b, A, B, R> LexemeParser<'a, 'b> for RuleBranch<A, B>
    where A: LexemeParser<'a, 'b, Result=R>,
          B: LexemeParser<'a, 'b, Result=R>,
{
    type Result = R;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let begin = cursor.index;
        let mut err = match self.0.parse(cursor) {
            Ok(v) => return Ok(v),
            Err(e) => e,
        };
        cursor.index = begin;
        match self.1.parse(cursor) {
            Ok(v) => Ok(v),
            Err(e) => {
                err.append(e);
                Err(err)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRepeat<A, B>(pub A, pub B);

use std::fmt::Debug;

impl<'a, 'b, A, B> LexemeParser<'a, 'b> for RuleRepeat<A, B>
    where A: LexemeParser<'a, 'b> + Debug,
          A::Result: Debug,
          B: NumRange<usize>,
{
    type Result = Vec<A::Result>;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
//        #[cfg(test)] trace!("Starting parsing repeating of {:?}", self.0);
        let mut result = match self.1.get_max() {
            Some(end) => Vec::with_capacity(end),
            None => Vec::new(),
        };
        let max = self.1.get_max();
        'parsing_cycle: for i in 1.. {
            if match max {
                Some(m) => i >= m,
                None => false,
            } { break 'parsing_cycle; }
//            #[cfg(test)] trace!("  Iteration #{}", _i);
            let begin = cursor.index;
            match self.0.parse(cursor) {
                Ok(v) => {
//                    #[cfg(test)] trace!("    Success! Got: {:?}", v);
                    result.push(v)
                }
                Err(e) => {
//                    #[cfg(test)] trace!("    Error! Checking count of success results");
                    if self.1.is_contains(&result.len()) {
//                        #[cfg(test)] trace!("      {} contains in {:?}", result.len(), self.1);
                        cursor.index = begin;
                        break 'parsing_cycle;
                    } else {
//                        #[cfg(test)] trace!("      {} not contains in {:?}", result.len(), self.1);
//                        #[cfg(test)] trace!("  Returning error {:?}", e);
                        return Err(e);
                    }
                }
            }
        }
//        #[cfg(test)] trace!("  Returning result {:?}", result);
        Ok(result)
    }
}
