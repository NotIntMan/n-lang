//! Набор структур для отображения элемента лексического разбора

use std::fmt;
use helpers::Assertion;
use super::*;
use parser_basics::Identifier;

/// Тип токена
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    /// Конец ввода. Этот тип токена генерируется сканером после окончания чтения ввода.
    EndOfInput,
    /// Пробел. Генерируется batcher'ом при нахождении группы пробельных символов, но игнорируется сканером.
    Whitespace,
    /// Числовой литерал. Генерируется сканером при нахождении группы цифр.
    NumberLiteral {
        /// Стоит ли перед числов знак минуса
        negative: bool,
        /// Имеет ли число дробную часть
        fractional: bool,
        /// Базис системы счисления
        radix: u32,
        /// Приблизительное значение найденного числа
        approx_value: f64,
    },
    /**
        Стрововый литерал. Генерируется сканером при нахождении символа кавычек (`"`).

        Между кавычками могут быть заключены любые символы, включая символы экранирования
        (символы, обозначенные префиксным обратным слэшем (`\`), кроме символа кавычек.
    */
    StringLiteral {
        length: u32,
    },
    /**
        Литерал выражения. Генерируется сканером при нахождения опострофа (`'`).

        Используется для обозначения регулярных выражений, дат и пр.
        Синтаксически, за исключением обозначения края опострофом вместо кавычек,
        эквивалентен строковому литералу.
    */
    BracedExpressionLiteral {
        length: u32,
    },
    /**
        Словестный литерал. Генерируется сканером при нахождении группы букв, цифр и символа `_`.

        Может быть использован для разбора идентификаторов и ключевых слов.
        Не может начинаться с чего-то, кроме буквы.
    */
    Word,
    /// Группа символов. Генерируется сканером при нахождении специального символа.
    SymbolGroup,
}

/// Урезанное отображение типа токена в его тип без прочей информации
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKindLess {
    EndOfInput,
    Whitespace,
    NumberLiteral,
    StringLiteral,
    BracedExpressionLiteral,
    Word,
    SymbolGroup,
}

impl fmt::Display for TokenKindLess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            &TokenKindLess::EndOfInput => "end of input",
            &TokenKindLess::Whitespace => "whitespace",
            &TokenKindLess::NumberLiteral => "number literal",
            &TokenKindLess::StringLiteral => "string literal",
            &TokenKindLess::BracedExpressionLiteral => "braced expression literal",
            &TokenKindLess::Word => "word",
            &TokenKindLess::SymbolGroup => "symbol group",
        };
        write!(f, "{}", text)
    }
}

/// Токен. Содержит информацию о своём типе, местоположении и тексте элемента, который отображает.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub pos: SymbolPosition,
}

impl<'a> Assertion for Token<'a> {
    fn assert(&self, other: &Self) {
        assert_eq!(self.kind, other.kind);
        assert_eq!(self.text, other.text);
    }
}

impl TokenKind {
    /// Подсказывает сканеру какие токены стоит игнорировать
    #[inline]
    pub fn is_must_not_be_ignored(&self) -> bool {
        self != &TokenKind::Whitespace
    }
    /// Подсказывает сканеру какие токены завершают его работу
    #[inline]
    pub fn is_end(&self) -> bool {
        self == &TokenKind::EndOfInput
    }
    /// Отображает тип токена в тип без прочей информации
    #[inline]
    pub fn less(&self) -> TokenKindLess {
        match self {
            &TokenKind::EndOfInput => TokenKindLess::EndOfInput,
            &TokenKind::Whitespace => TokenKindLess::Whitespace,
            &TokenKind::NumberLiteral {
                negative: _,
                fractional: _,
                radix: _,
                approx_value: _,
            } => TokenKindLess::NumberLiteral,
            &TokenKind::StringLiteral {
                length: _,
            } => TokenKindLess::StringLiteral,
            &TokenKind::BracedExpressionLiteral {
                length: _,
            } => TokenKindLess::BracedExpressionLiteral,
            &TokenKind::Word => TokenKindLess::Word,
            &TokenKind::SymbolGroup => TokenKindLess::SymbolGroup,
        }
    }
    /**
        Конструирует новый строковый `TokenKind` из `TokenKindLess` и агрумента `length`.
        Очевидно, успех достигается в случае, если `kind` имеет значение `StringLiteral` или `BracedExpressionLiteral`.
        В прочих случаях, `kind` приравнивается к `StringLiteral`.
    */
    #[inline]
    pub fn new_string_literal(kind: TokenKindLess, length: u32) -> Self {
        match kind {
            TokenKindLess::BracedExpressionLiteral => TokenKind::BracedExpressionLiteral { length },
            _ => TokenKind::StringLiteral { length },
        }
    }
}

impl<'a> Token<'a> {
    /// Создаёт новый токен из переданных данных
    #[allow(dead_code)]
    pub fn new(kind: TokenKind, text: &'a str, pos: SymbolPosition) -> Self {
        Self { kind, text, pos }
    }
    /// Метод, полезный для тектирования. Возвращает то же, что и метод `new`, но обёрнутое в `Some(Ok(_))`.
    #[allow(dead_code)]
    pub fn new_wrapped(kind: TokenKind, text: &'a str, pos: SymbolPosition) -> Option<ScannerItem<'a>> {
        Some(Ok(Self::new(kind, text, pos)))
    }
    pub fn ident(&self) -> Identifier<'a> {
        Identifier::new(self.text, self.pos)
    }
    pub fn pos(&self) -> ItemPosition {
        self.pos.make_item_pos(self.text)
    }
}
