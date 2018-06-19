use lexeme_scanner::{
    TokenKind,
    ScannerErrorKind,
};
use lexeme_scanner::rules::number::is_number_begin;
use lexeme_scanner::rules::basics::*;

/// Совершает попытку разбора числового литерала.
/// Полностью соответствует спецификации числовых литералов языка.
pub fn parse_number_literal(input: &[u8]) -> Result<(TokenKind, f64, usize), (ScannerErrorKind, usize)> {
    let len = input.len();
    if !is_number_begin(input) {
        let got = extract_char(input, 0, "begin of a number")?;
        return Err((ScannerErrorKind::must_be_got("begin of a number", got), 0));
    }
    let (
        negative,
        mut result,
    ) = if (len > 0) && ((input[0] as char) == '-') {
        (true, 1)
    } else {
        (false, 0)
    };
    let mut fractional = false;
    let radix = if extract_char(input, result, "a number")? == '0' {
        result += 1;
        if result >= len {
            return Ok((TokenKind::NumberLiteral { negative, radix: 10, fractional: false, approx_value: 0.0 }, 0.0, result));
        }
        match input[result] as char {
            'x' => {
                result += 1;
                16
            }
            'o' => {
                result += 1;
                8
            }
            'b' => {
                result += 1;
                2
            }
            '.' => {
                assert_pred(input, result + 1,
                            |c| c.is_digit(10), "a decimal digit")?;
                result += 1;
                fractional = true;
                10
            }
            _ => 8,
        }
    } else { 10 };
    let mut value = 0.0;
    let mut fractional_coefficient = 1.0;
    'parse_cycle: loop {
        if result >= len {
            break 'parse_cycle;
        }
        let c = input[result] as char;
        if c.is_digit(16) {
            match c.to_digit(radix) {
                Some(v) => {
                    result += 1;
                    if fractional {
                        fractional_coefficient /= f64::from(radix);
                        value += f64::from(v) * fractional_coefficient;
                    } else {
                        value *= f64::from(radix);
                        value += f64::from(v);
                    }
                    continue 'parse_cycle;
                }
                None => return Err((ScannerErrorKind::NotInRadix(c, radix), result)),
            }
        }
        if c == '.' {
            if !fractional && ((result + 1) < len) && (input[result + 1] as char).is_digit(16) {
                fractional = true;
                result += 1;
                continue 'parse_cycle;
            }
        }
        break 'parse_cycle;
    }
    if negative {
        value *= -1.0;
    }
    Ok((TokenKind::NumberLiteral { negative, fractional, radix, approx_value: value }, value, result))
}

/// Совершает попытку разбора числового литерала.
/// Полностью соответствует спецификации числовых литералов языка.
#[inline]
pub fn parse_number_literal_str(input: &str) -> Result<(TokenKind, f64, usize), (ScannerErrorKind, usize)> {
    parse_number_literal(input.as_bytes())
}

/// Совершает попытку разбора целочисленного положительного литерала.
/// Полностью соответствует спецификации числовых литералов языка.
#[inline]
pub fn parse_integer_literal(input: &str) -> Option<u32> {
    let parse_result = parse_number_literal_str(input).ok()?.1;
    if parse_result.is_sign_positive() && (parse_result.floor() == parse_result.ceil()) {
        Some(parse_result as u32)
    } else {
        None
    }
}

#[test]
fn number_parses_correctly() {
    assert_eq!(parse_number_literal_str("0.4").unwrap().1, 0.4);
    assert_eq!(parse_number_literal_str("00.4").unwrap().1, 0.5);
    assert_eq!(parse_number_literal_str("-2").unwrap().1, -2.0);
    assert_eq!(parse_number_literal_str("-0xA").unwrap().1, -10.0);
    assert_eq!(parse_number_literal_str("-0xa").unwrap().1, -10.0);
}
