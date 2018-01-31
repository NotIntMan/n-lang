//! Набор структур, служащиз для отображения ошибки сканирования

use std::fmt::{
    Display,
    Result,
    Formatter,
};

use super::*;

/**
    Тип ошибки скинарования

    Перечисление - весьма полезный инструмент, который нашёл своё применение и здесь.
    Каждый его вариант содержит минимальный и необходимый набор данных для отображения
    информации о произошедней ошибке во время сканирования текста.
*/
#[derive(Debug, Clone, PartialEq)]
pub enum ScannerErrorKind {
    /**
        Этот тип ошибки возникает, когда сканером ожидается какой-то определённый символ,
        а в тексте на той же позиции содержится другой.

        Выводит сообщение типа `Expected: Some("<"), got: Some("%").`
    */
    ExpectedGot(char, Option<char>),
    /**
        Этот тип ошибки возникает, когда сканер использует тестирующую функцию для определения
        принадлежности символа к определённой категории и символ этот тест не проходит.

        Выводит сообщение типа `Symbol must be a decimal digit, got: Some("H").`
    */
    MustBeGot(String, Option<char>),
    /**
        Этот тип ошибки возникает, когда сканер не готов закончить разбор из-за окончания
        ввода, однако ввод был окончен.

        Выводит сообщение `Unexpected end of input.`
    */
    UnexpectedEnd,
    /**
        Этот тип ошибки возникает, когда сканер, в процессе разбора числа, находит цифру
        равную, либо большую базису системы счисления числа. Например,
        `A` при разборе десятичных чисел или `9` при разборе восьмеричных.

        Выводит сообщение типа `Digit "c" is not in 2-based radix`
    */
    NotInRadix(char, u32),
}

/// Ошибка сканирования. Содержит в себе сообщение об ошибке и указание места в тексте, где была сгенерирована эта ошибка.
#[derive(Debug, Clone, PartialEq)]
pub struct ScannerError {
    pub kind: ScannerErrorKind,
    pub pos: SymbolPosition,
}

/// В реализации `Display` реализовано отображение типа ошибки в её сообщение.
impl Display for ScannerErrorKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &ScannerErrorKind::ExpectedGot(ref e, ref g) => write!(f, "expected: Some({:?}), got: {:?}", e, g),
            &ScannerErrorKind::MustBeGot(ref m, ref g) => write!(f, "symbol must be {}, got: {:?}", m, g),
            &ScannerErrorKind::UnexpectedEnd => write!(f, "unexpected end of input"),
            &ScannerErrorKind::NotInRadix(c, r) => write!(f, "digit {:?} is not in {}-based radix", c, r),
        }
    }
}

/// В реализации `Display` реализовано отображение ошибки в её сообщение.
impl Display for ScannerError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error: {} on {}", self.kind, self.pos)
    }
}
