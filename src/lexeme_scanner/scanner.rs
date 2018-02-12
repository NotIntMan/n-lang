//! Сканер. Получает строку на ввод и реализует итерируемый механизм сканирования лексем в полученонй строке.

use std::iter::FromIterator;

use super::*;
use super::rules::scan;

/// Сканер. Получает строку на ввод и реализует итерируемый механизм сканирования лексем в полученонй строке.
pub struct Scanner<'a> {
    source: &'a str,
    position: SymbolPosition,
    finished: bool,
}

impl<'a> Iterator for Scanner<'a>
    where Scanner<'a>: 'a {
    type Item = ScannerItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        loop {
            let (scan_result, len) = match scan(self.source.as_bytes()) {
                Ok((kind, len)) => (Ok(kind), len),
                Err((kind, len)) => (Err(kind), len),
            };
            let text = &self.source[..len];
            let begin = self.position.clone();
            self.position.step_str(text);
            let kind = match scan_result {
                Ok(kind) => kind,
                Err(kind) => {
                    self.finished = true;
                    return Some(Err(ScannerError { kind, pos: self.position.clone() }));
                },
            };
            self.source = &self.source[len..];
            if kind.is_end() {
                self.finished = true;
            }
            if kind.is_must_not_be_ignored() {
                return Some(Ok(Token { kind, pos: begin, text }));
            }
        }
    }
}

impl<'a> Scanner<'a> {
    /// Создаёт новый сканер
    #[inline]
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: SymbolPosition::default(),
            finished: false,
        }
    }
    /// Пробует собрать все лексемы (токены) в единый буфер.
    /// Если встречает ошибку, возвращает её.
    #[inline]
    pub fn into_buffer(self) -> Result<Vec<Token<'a>>, ScannerError> {
        Result::from_iter(self)
    }
    /**
        Сканирование

        Производит сканирование и складывает все лексемы в PerfectBuffer.
        Является композицией методов `new` и `into_buffer`.
    */
    #[inline]
    pub fn scan(source: &'a str) -> Result<Vec<Token<'a>>, ScannerError> {
        Scanner::new(source).into_buffer()
    }
}
