//! Сканер. Получает строку на ввод и реализует итерируемый механизм сканирования лексем в полученонй строке.

use std::str::Chars;
use std::iter::FromIterator;

use super::*;
use super::eaters::scan;
use super::super::helpers::iter_buffer::{
    IterBuffer,
    PerfectBuffer,
    IterBufferCursor,
};

pub type ScannerCursor<'a, 'b> = IterBufferCursor<'a, Chars<'b>>;

/// Сканер. Получает строку на ввод и реализует итерируемый механизм сканирования лексем в полученонй строке.
pub struct Scanner<'a> {
    source: &'a str,
    buffer: IterBuffer<Chars<'a>>,
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
            let begin = self.position.clone();
            let mut cursor = self.buffer.cursor(begin.offset);
            let scan_result = scan(&mut cursor);
            let len = cursor.index - begin.offset;
            let text = &self.source[..len];
            self.position.step_str(text);
            let kind = match scan_result {
                Ok(kind) => kind,
                Err(kind) => {
                    self.finished = true;
                    return Some(Err(ScannerError { kind, pos: self.position.clone() }));
                }
            };
            self.source = &self.source[len..];
            let pos = ItemPosition { begin, end: self.position.clone() };
            if kind.is_end() {
                self.finished = true;
            }
            if kind.is_must_not_be_ignored() {
                return Some(Ok(Token { kind, pos, text }));
            }
        }
    }
}

impl<'a> Scanner<'a> {
    /// Создаёт новый сканер
    #[inline]
    pub fn new(source: &'a str) -> Self {
        let buffer = IterBuffer::new(source.chars());
        Self {
            source,
            buffer,
            position: SymbolPosition::default(),
            finished: false,
        }
    }
    /// Пробует собрать все лексемы (токены) в единый буфер.
    /// Если встречает ошибку, возвращает её.
    #[inline]
    pub fn into_buffer(self) -> Result<PerfectBuffer<Token<'a>>, ScannerError> {
        Result::from_iter(self)
    }
    /**
        Сканирование

        Производит сканирование и складывает все лексемы в PerfectBuffer.
        Является композицией методов `new` и `into_buffer`.
    */
    #[inline]
    pub fn scan(source: &'a str) -> Result<PerfectBuffer<Token<'a>>, ScannerError> {
        Scanner::new(source).into_buffer()
    }
}
