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

        Выводит сообщение типа `expected: Some("<"), got: Some("%").`
    */
    ExpectedGot(char, char),
    /**
        Этот тип ошибки возникает, когда сканер использует тестирующую функцию для определения
        принадлежности символа к определённой категории и символ этот тест не проходит.

        Выводит сообщение типа `symbol must be a decimal digit, got: "H".`
    */
    MustBeGot(String, char),
    /**
        Этот тип ошибки возникает, когда сканер не готов закончить разбор из-за окончания
        ввода, однако ввод был окончен.

        Выводит сообщение `unexpected end of input.`
    */
    UnexpectedEnd(Option<String>),
    /**
        Этот тип ошибки возникает, когда сканер, в процессе разбора числа, находит цифру
        равную, либо большую базису системы счисления числа. Например,
        `A` при разборе десятичных чисел или `9` при разборе восьмеричных.

        Выводит сообщение типа `digit "c" is not in 2-based radix`
    */
    NotInRadix(char, u32),
    /**
        Этот тип ошибки возникает, когда сканер находит символ, который не подходит ни одному правилу.

        Выводит сообщение `unexpected input`.
    */
    UnexpectedInput(char),
}

/// Ошибка сканирования. Содержит в себе сообщение об ошибке и указание места в тексте, где была сгенерирована эта ошибка.
#[derive(Debug, Clone, PartialEq)]
pub struct ScannerError {
    pub kind: ScannerErrorKind,
    pub pos: SymbolPosition,
}

impl ScannerErrorKind {
    /// Конструирует новый `ScannerErrorKind::UnexpectedEnd` с сообщением о том, что ожидался символ
    #[inline]
    pub fn unexpected_end_expected_char(c: char) -> Self {
        ScannerErrorKind::UnexpectedEnd(Some(format!("{:?}", c)))
    }
    /// Конструирует новый `ScannerErrorKind::UnexpectedEnd` с данным сообщением об ожидании
    #[inline]
    pub fn unexpected_end_expected<S: Into<String>>(msg: S) -> Self {
        ScannerErrorKind::UnexpectedEnd(Some(msg.into()))
    }
    /// Конструирует новый `ScannerErrorKind::UnexpectedEnd` без сообщения
    #[inline]
    pub fn unexpected_end() -> Self {
        ScannerErrorKind::UnexpectedEnd(None)
    }
    #[inline]
    pub fn must_be_got<S: Into<String>>(must_be: S, got: char) -> Self {
        ScannerErrorKind::MustBeGot(must_be.into(), got)
    }
}

/// В реализации `Display` реализовано отображение типа ошибки в её сообщение.
impl Display for ScannerErrorKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &ScannerErrorKind::ExpectedGot(ref e, ref g) => write!(f, "expected: {:?}, got: {:?}", e, g),
            &ScannerErrorKind::MustBeGot(ref m, ref g) => write!(f, "symbol must be {}, got: {:?}", m, g),
            &ScannerErrorKind::UnexpectedEnd(ref e) => {
                write!(f, "unexpected end of input")?;
                if let &Some(ref c) = e {
                    write!(f, ", expected: {}", c)?;
                }
                Ok(())
            },
            &ScannerErrorKind::NotInRadix(ref c, ref r) => write!(f, "digit {:?} is not in {}-based radix", c, r),
            &ScannerErrorKind::UnexpectedInput(ref c) => write!(f, "unexpected input {:?}", c),
        }
    }
}

/// В реализации `Display` реализовано отображение ошибки в её сообщение.
impl Display for ScannerError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error: {} on {}", self.kind, self.pos)
    }
}
