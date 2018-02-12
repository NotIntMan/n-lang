use lexeme_scanner::{
    Token,
    TokenKind,
    TokenKindLess,
};
use parser_basics::{
    keyword,
    token,
    ParserInput,
    ParserErrorKind,
    ParserResult,
};
use super::*;

/// token(TokenKindLess::NumberLiteral)
parser_rule!(number_literal(i) -> Literal<'source> {
    do_parse!(i,
        token: apply!(token, TokenKindLess::NumberLiteral) >>
        ({
            let literal_type = match token.kind {
                TokenKind::NumberLiteral { negative, fractional, radix } => {
                    LiteralType::NumberLiteral { negative, fractional, radix }
                },
                _ => return i.err(ParserErrorKind::expected_got_kind(TokenKindLess::NumberLiteral, token.kind.less())),
            };
            Literal { literal_type, token: *token }
        })
    )
});

/// token(TokenKindLess::StringLiteral)
parser_rule!(string_literal(i) -> Literal<'source> {
    do_parse!(i,
        token: apply!(token, TokenKindLess::StringLiteral) >>
        ({
            let literal_type = match token.kind {
                TokenKind::StringLiteral { length } => {
                    LiteralType::StringLiteral { length }
                },
                _ => return i.err(ParserErrorKind::expected_got_kind(TokenKindLess::StringLiteral, token.kind.less())),
            };
            Literal { literal_type, token: *token }
        })
    )
});

/// token(TokenKindLess::BracedExpressionLiteral)
parser_rule!(braced_literal(i) -> Literal<'source> {
    do_parse!(i,
        token: apply!(token, TokenKindLess::BracedExpressionLiteral) >>
        ({
            let literal_type = match token.kind {
                TokenKind::BracedExpressionLiteral { length } => {
                    LiteralType::BracedExpressionLiteral { length }
                },
                _ => return i.err(ParserErrorKind::expected_got_kind(TokenKindLess::BracedExpressionLiteral, token.kind.less())),
            };
            Literal { literal_type, token: *token }
        })
    )
});

/// "true" | "false" | "null"
parser_rule!(keyword_literal(i) -> Literal<'source> {
    do_parse!(i,
        x: alt!(
            apply!(keyword, "true") => { |t| (t, KeywordLiteralType::True) } |
            apply!(keyword, "false") => { |t| (t, KeywordLiteralType::False) } |
            apply!(keyword, "null") => { |t| (t, KeywordLiteralType::Null) }
        ) >>
        ({
            let (token, literal_type) = x;
            Literal { literal_type: LiteralType::KeywordLiteral(literal_type), token: *token }
        })
    )
});

/// Функция, выполняющая разбор литералов
pub fn literal<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Literal<'source>> {
    alt!(input, number_literal | string_literal | braced_literal | keyword_literal)
}

#[test]
fn number_literals_parses_correctly() {
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
    {
        let tokens = Scanner::scan("-2")
            .expect("Scan result with no error");
        assert_eq!(
            parse(tokens.as_slice(), literal)
                .expect("Parse result with no error")
                .literal_type,
            LiteralType::NumberLiteral {
                negative: true,
                fractional: false,
                radix: 10,
            }
        );
    }
    {
        let tokens = Scanner::scan("0b101.1")
            .expect("Scan result with no error");
        assert_eq!(
            parse(tokens.as_slice(), literal)
                .expect("Parse result with no error")
                .literal_type,
            LiteralType::NumberLiteral {
                negative: false,
                fractional: true,
                radix: 2,
            }
        );
    }
}

#[test]
fn string_and_braced_literals_parses_correctly() {
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
    {
        let tokens = Scanner::scan("\"azaz\"")
            .expect("Scan result with no error");
        assert_eq!(
            parse(tokens.as_slice(), literal)
                .expect("Parse result with no error")
                .literal_type,
            LiteralType::StringLiteral { length: 4 }
        );
    }
    {
        let tokens = Scanner::scan("'can\nzas'")
            .expect("Scan result with no error");
        assert_eq!(
            parse(tokens.as_slice(), literal)
                .expect("Parse result with no error")
                .literal_type,
            LiteralType::BracedExpressionLiteral { length: 7 }
        );
    }
}

#[test]
fn keyword_literals_parses_correctly() {
    use lexeme_scanner::Scanner;
    use parser_basics::parse;
    {
        let tokens = Scanner::scan("true")
            .expect("Scan result with no error");
        assert_eq!(
            parse(tokens.as_slice(), literal)
                .expect("Parse result with no error")
                .literal_type,
            LiteralType::KeywordLiteral(KeywordLiteralType::True)
        );
    }
    {
        let tokens = Scanner::scan("false")
            .expect("Scan result with no error");
        assert_eq!(
            parse(tokens.as_slice(), literal)
                .expect("Parse result with no error")
                .literal_type,
            LiteralType::KeywordLiteral(KeywordLiteralType::False)
        );
    }
    {
        let tokens = Scanner::scan("null")
            .expect("Scan result with no error");
        assert_eq!(
            parse(tokens.as_slice(), literal)
                .expect("Parse result with no error")
                .literal_type,
            LiteralType::KeywordLiteral(KeywordLiteralType::Null)
        );
    }
}
