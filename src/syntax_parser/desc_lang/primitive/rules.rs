use syntax_parser::basics::{
    BasicKeyword,
    BasicUSizeLiteral,
//    constants,
    LexemeCursor,
    LexemeParser,
    LexemeParserResult,
    TemplateWrap,
};

use super::definitions::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberFlags;

impl<'a, 'b> LexemeParser<'a, 'b> for NumberFlags {
    type Result = (bool, bool);
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
        let unsigned = parse_option!(cursor: cursor, BasicKeyword("unsigned").parse(cursor))?.is_some();
        let zerofill = parse_option!(cursor: cursor, BasicKeyword("zerofill").parse(cursor))?.is_some();
        Ok((unsigned, zerofill))
    }
}

pub fn integer<'a, 'b>(cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<NumberType> {
    let integer_type = parse_branch!(
        cursor: cursor,
        parse_sequence!(BasicKeyword("tiny").parse(cursor), return Ok(IntegerType::Tiny)),
        parse_sequence!(BasicKeyword("small").parse(cursor), return Ok(IntegerType::Small)),
        parse_sequence!(BasicKeyword("medium").parse(cursor), return Ok(IntegerType::Medium)),
        parse_sequence!(BasicKeyword("big").parse(cursor), return Ok(IntegerType::Big)),
        parse_sequence!(return Ok(IntegerType::Normal)),
    )?;
    let (unsigned, zerofill) = NumberFlags.parse(cursor)?;
    Ok(NumberType::Integer { integer_type, unsigned, zerofill })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberTypeParser;

impl<'a, 'b> LexemeParser<'a, 'b> for NumberTypeParser {
    type Result = NumberType;
    fn parse(&self, cursor: &mut LexemeCursor<'a, 'b>) -> LexemeParserResult<Self::Result> {
//        parse_sequence!(
//            constants::OPENING_ROUND_BRACKET.parse(cursor),
//            let
//            constants::CLOSING_ROUND_BRACKET.parse(cursor),
//        )
//        let size = parse_option!(
//            cursor: cursor,
//
//        );
//        parse_sequence!(
//            BasicKeyword("decimal").parse(cursor),
////            let size = parse
//        );
        parse_branch!(
            cursor: cursor,
            // Bit
            parse_sequence!(
                BasicKeyword("bit").parse(cursor),
                let size = parse_option!(cursor: cursor, TemplateWrap::round(BasicUSizeLiteral).parse(cursor)),
                return Ok(NumberType::Bit {size})
            ),
            // Boolean
            parse_sequence!(
                BasicKeyword("boolean").parse(cursor),
                return Ok(NumberType::Boolean)
            ),
            // Integer
            integer(cursor),
            // Decimal

        );
        unimplemented!()
    }
}
