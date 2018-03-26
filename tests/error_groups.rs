extern crate n_lang;

use n_lang::helpers::group::{
    Appendable,
    Group,
};
use n_lang::lexeme_scanner::TokenKindLess;
use n_lang::parser_basics::{
    new_error_without_pos,
    ParserErrorItem,
    ParserErrorKind,
    ParserErrorTokenInfo,
};

#[test]
fn simple_errors_groups_correctly() {
    let mut group = new_error_without_pos(
        ParserErrorKind::expected_got(
            ParserErrorTokenInfo::from_kind(TokenKindLess::NumberLiteral),
            ParserErrorTokenInfo::from_kind(TokenKindLess::Word),
        )
    );
    group.append(new_error_without_pos(
        ParserErrorKind::expected_got(
            ParserErrorTokenInfo::from_kind(TokenKindLess::StringLiteral),
            ParserErrorTokenInfo::from_kind(TokenKindLess::Word),
        )
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
    let mut group = new_error_without_pos(
        ParserErrorKind::expected_got(
            ParserErrorTokenInfo::from_kind(TokenKindLess::NumberLiteral),
            ParserErrorTokenInfo::from_kind(TokenKindLess::Word),
        )
    );
    group.append(new_error_without_pos(
        ParserErrorKind::expected_got(
            ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::Word, "final"),
            ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::Word, "end"),
        )
    ));
    assert_eq!(group, Group::One(ParserErrorItem {
        kind: ParserErrorKind::ExpectedGot(
            Group::Many(vec![
                ParserErrorTokenInfo::from_kind(TokenKindLess::NumberLiteral),
                ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::Word, "final"),
            ]),
            ParserErrorTokenInfo::from_kind_and_desc(TokenKindLess::Word, "end"),
        ),
        pos: None,
    }));
}

