use std::fmt::{
    Write,
    Result,
    Display,
};
use std::ops::{
    DivAssign,
    SubAssign,
    Rem,
};

pub fn write_pad_left<W: Write, D: Display>(w: &mut W, value: D, length: usize) -> Result {
    let value_string = format!("{}", value);
    for _ in value_string.len()..length {
        write!(w, " ")?;
    }
    write!(w, "{}", value_string)?;
    Ok(())
}

pub fn write_line_numbers_columns_row<W: Write>(w: &mut W, number_length: usize, line_number: Option<usize>) -> Result {
    write!(w, " ")?;
    match line_number {
        Some(line_number) => write_pad_left(w, line_number, number_length)?,
        None => {
            for _ in 0..number_length {
                write!(w, " ")?;
            }
        }
    }
    write!(w, " | ")
}

#[inline]
pub fn generic_unsigned_length<T, Z, B>(mut value: T, zero: Z, basis: B) -> usize
    where T: DivAssign<B>
    + SubAssign<<T as Rem<B>>::Output>
    + Rem<B>
    + PartialOrd<Z>
    + Clone,
          B: Clone,
{
    let mut result = 0;
    while value.clone() > zero {
        result += 1;
        value -= value.clone() % basis.clone();
        ;
        value /= basis.clone();
    }
    result
}

#[inline]
pub fn decimal_unsigned_length<T>(value: T) -> usize
    where T: DivAssign
    + SubAssign<<T as Rem>::Output>
    + Rem
    + PartialOrd
    + Clone
    + From<u8>
{
    generic_unsigned_length(value, T::from(0), T::from(10))
}

pub fn left_padding_size(line: &str) -> Option<usize> {
    let mut result = 1;
    for c in line.chars() {
        if c.is_whitespace() {
            result += 1;
        } else {
            return Some(result);
        }
    }
    None
}

pub fn write_pointer_line<W: Write>(w: &mut W, line: &str, number_length: usize, begin: usize, end: usize) -> Result {
    let begin = match left_padding_size(line) {
        Some(padding) => padding.max(begin),
        None => return Ok(()),
    };
    if begin <= end {
        write_line_numbers_columns_row(w, number_length, None)?;
        for _ in 1..begin {
            write!(w, " ")?;
        }
        for _ in begin..=end {
            write!(w, "^")?;
        }
        writeln!(w, "")?;
    }
    Ok(())
}

pub fn display<F: Fn(&mut String) -> Result>(func: F) -> String {
    let mut result = String::new();
    let _ = func(&mut result);
    result
}
