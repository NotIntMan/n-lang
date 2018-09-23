use lexeme_scanner::{
    Token,
    TokenKind,
    TokenKindLess,
};
use parser_basics::{
    keyword,
    ParserErrorKind,
    ParserErrorTokenInfo,
    ParserInput,
    ParserResult,
    token,
};
use super::*;

/// token(TokenKindLess::NumberLiteral)
parser_rule!(number_literal(i) -> LiteralAST<'source> {
    do_parse!(i,
        token: apply!(token, TokenKindLess::NumberLiteral) >>
        ({
            let literal_type = match token.kind {
                TokenKind::NumberLiteral { negative, fractional, radix, approx_value } => {
                    LiteralType::NumberLiteral { negative, fractional, radix, approx_value }
                },
                other => return i.err(ParserErrorKind::expected_got(
                    ParserErrorTokenInfo::from_kind(TokenKindLess::NumberLiteral),
                    ParserErrorTokenInfo::from_kind(other.less()),
                )),
            };
            LiteralAST { literal_type, text: token.ident(), pos: token.pos() }
        })
    )
});

/// token(TokenKindLess::StringLiteral)
parser_rule!(string_literal(i) -> LiteralAST<'source> {
    do_parse!(i,
        token: apply!(token, TokenKindLess::StringLiteral) >>
        ({
            let literal_type = match token.kind {
                TokenKind::StringLiteral { length } => {
                    LiteralType::StringLiteral { length }
                },
                other => return i.err(ParserErrorKind::expected_got(
                    ParserErrorTokenInfo::from_kind(TokenKindLess::StringLiteral),
                    ParserErrorTokenInfo::from_kind(other.less()),
                )),
            };
            LiteralAST { literal_type, text: token.ident(), pos: token.pos() }
        })
    )
});

/// token(TokenKindLess::BracedExpressionLiteral)
parser_rule!(braced_literal(i) -> LiteralAST<'source> {
    do_parse!(i,
        token: apply!(token, TokenKindLess::BracedExpressionLiteral) >>
        ({
            let literal_type = match token.kind {
                TokenKind::BracedExpressionLiteral { length } => {
                    LiteralType::BracedExpressionLiteral { length }
                },
                other => return i.err(ParserErrorKind::expected_got(
                    ParserErrorTokenInfo::from_kind(TokenKindLess::BracedExpressionLiteral),
                    ParserErrorTokenInfo::from_kind(other.less()),
                )),
            };
            LiteralAST { literal_type, text: token.ident(), pos: token.pos() }
        })
    )
});

/// "true" | "false" | "null"
parser_rule!(keyword_literal(i) -> LiteralAST<'source> {
    do_parse!(i,
        x: alt!(
            apply!(keyword, "true") => { |t| (t, KeywordLiteralType::True) } |
            apply!(keyword, "false") => { |t| (t, KeywordLiteralType::False) } |
            apply!(keyword, "null") => { |t| (t, KeywordLiteralType::Null) }
        ) >>
        ({
            let (token, literal_type) = x;
            LiteralAST { literal_type: LiteralType::KeywordLiteral(literal_type), text: token.ident(), pos: token.pos() }
        })
    )
});

/// Функция, выполняющая разбор литералов
pub fn literal<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, LiteralAST<'source>> {
    alt!(input, number_literal | string_literal | braced_literal | keyword_literal)
}

