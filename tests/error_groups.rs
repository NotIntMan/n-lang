extern crate n_transpiler;

use n_transpiler::helpers::group::{
    Appendable,
    Group,
};
use n_transpiler::lexeme_scanner::TokenKindLess;
use n_transpiler::parser_basics::{
    ParserError,
    ParserErrorItem,
    ParserErrorKind,
    ParserErrorTokenInfo,
};

#[test]
fn simple_errors_groups_correctly() {
    let mut group = ParserError::new_without_pos(
        ParserErrorKind::expected_got_kind(TokenKindLess::NumberLiteral, TokenKindLess::Word)
    );
    group.append(ParserError::new_without_pos(
        ParserErrorKind::expected_got_kind(TokenKindLess::StringLiteral, TokenKindLess::Word)
    ));
    assert_eq!(group, Group::One(ParserErrorItem {
        kind: ParserErrorKind::ExpectedGot(
            Group::Many(vec![
                ParserErrorTokenInfo::from_kind(TokenKindLess::NumberLiteral),
                ParserErrorTokenInfo::from_kind(TokenKindLess::StringLiteral),
            ]),
            ParserErrorTokenInfo::from_kind(TokenKindLess::Word),
        ),
        pos: None,
    }));
}

#[test]
fn simple_errors_with_dif_level_of_knowledge_groups_correctly() {
    let mut group = ParserError::new_without_pos(
        ParserErrorKind::expected_got_kind(TokenKindLess::NumberLiteral, TokenKindLess::Word)
    );
    group.append(ParserError::new_without_pos(
        ParserErrorKind::expected_got_kind_text(TokenKindLess::Word, "final", TokenKindLess::Word, "end")
    ));
    assert_eq!(group, Group::One(ParserErrorItem {
        kind: ParserErrorKind::ExpectedGot(
            Group::Many(vec![
                ParserErrorTokenInfo::from_kind(TokenKindLess::NumberLiteral),
                ParserErrorTokenInfo {
                    kind: Some(TokenKindLess::Word),
                    desc: Some("final".to_string()),
                }
            ]),
            ParserErrorTokenInfo {
                kind: Some(TokenKindLess::Word),
                desc: Some("end".to_string()),
            }
        ),
        pos: None,
    }));
}

