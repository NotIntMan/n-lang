//! Набор структур для отображения элемента лексического разбора

use super::*;

/// Тип токена
#[derive(Debug, Clone, PartialEq, Eq)]
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
    },
    /**
        Стрововый литерал. Генерируется сканером при нахождении символа кавычек (`"`).

        Между кавычками могут быть заключены любые символы, включая символы экранирования
        (символы, обозначенные префиксным обратным слэшем (`\`), кроме символа кавычек.
    */
    StringLiteral,
    /**
        Литерал выражения. Генерируется сканером при нахождения опострофа (`'`).

        Используется для обозначения регулярных выражений, дат и пр.
        Синтаксически, за исключением обозначения края опострофом вместо кавычек,
        эквивалентен строковому литералу.
    */
    BracedExpressionLiteral,
    /**
        Словестный литерал. Генерируется сканером при нахождении группы букв, цифр и символа `_`.

        Может быть использован для разбора идентификаторов и ключевых слов.
        Не может начинаться с чего-то, кроме буквы.
    */
    Word,
    /// Группа символов. Генерируется сканером при нахождении специального символа.
    SymbolGroup,
}

/// Токен. Содержит информацию о своём типе, местоположении и тексте элемента, который отображает.
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub pos: ItemPosition,
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
    /// Проверяет является ли тип токены числовым литералом
    #[inline]
    pub fn is_number(&self) -> bool {
        match self {
            &TokenKind::NumberLiteral { negative: _, fractional: _, radix: _ } => true,
            _ => false,
        }
    }
}

impl<'a> Token<'a> {
    /// Создаёт новый токен из переданных данных. Вычисляет позицию окончания автоматически.
    #[allow(dead_code)]
    pub fn new(kind: TokenKind, text: &'a str, begin: SymbolPosition) -> Self {
        let mut end = begin.clone();
        end.step_str(text);
        Self { kind, text, pos: ItemPosition { begin, end } }
    }
    /// Метод, полезный для тектирования. Возвращает то же, что и метод `new`, но обёрнутое в `Some(Ok(_))`.
    #[allow(dead_code)]
    pub fn new_wrapped(kind: TokenKind, text: &'a str, begin: SymbolPosition) -> Option<ScannerItem<'a>> {
        Some(Ok(Self::new(kind, text, begin)))
    }
}
