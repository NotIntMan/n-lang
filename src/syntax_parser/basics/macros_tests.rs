use helpers::num_range::NumRange;

use lexeme_scanner::{
    Scanner,
    TokenKind,
};

use super::{
    LexemeCursor,
    LexemeExact,
    LexemeParser,
};

#[test]
fn brainfuck_counter_test() {
    use env_logger::try_init;
    let _ = try_init();
    let mut buf = Scanner::scan("++-+++----")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    let plus = LexemeExact(TokenKind::SymbolGroup, "+");
    let minus = LexemeExact(TokenKind::SymbolGroup, "-");
    let inc = |c: &mut LexemeCursor| parse_sequence!(
        plus.parse(c),
        return Ok(1)
    );
    let dec = |c: &mut LexemeCursor| parse_sequence!(
        minus.parse(c),
        return Ok(-1)
    );
    let one = |c: &mut LexemeCursor| parse_branch!(
        cursor: c,
        inc(c),
        dec(c),
    );
    let many = |c: &mut LexemeCursor| parse_repeat!(
        cursor: c,
        range: ..4,
        one(c)
    );
    let fold = |c: &mut LexemeCursor| parse_sequence!(
        let m = many(c),
        return {
            let mut sum = 0;
            for i in m {
                sum += i;
            }
            Ok(sum)
        }
    );
    let double_fold = |c: &mut LexemeCursor| parse_sequence!(
        let a = fold(c),
        let b = fold(c),
        return { Ok(a + b) }
    );
    assert_eq!(
        double_fold(&mut cursor)
            .expect("Parsing result with no error"),
        4
    );
}

#[test]
fn brainfuck_counter_composition_test() {
    use env_logger::try_init;
    let _ = try_init();
    let mut buf = Scanner::scan("++-+++----")
        .expect("Scanning result with no error");
    let mut cursor = buf.cursor(0);
    let fold = |c: &mut LexemeCursor| parse_sequence!(
        let m = parse_repeat!(
            cursor: c,
            range: ..4,
            parse_branch!(
                cursor: c,
                parse_sequence!(
                    LexemeExact(TokenKind::SymbolGroup, "+").parse(c),
                    return Ok(1)
                ),
                parse_sequence!(
                    LexemeExact(TokenKind::SymbolGroup, "-").parse(c),
                    return Ok(-1)
                ),
            )
        ),
        return {
            let mut sum = 0;
            for i in m {
                sum += i;
            }
            Ok(sum)
        }
    );
    let double_fold = |c: &mut LexemeCursor| parse_sequence!(
        let a = fold(c),
        let b = fold(c),
        return { Ok(a + b) }
    );
    assert_eq!(
        double_fold(&mut cursor)
            .expect("Parsing result with no error"),
        4
    );
}
