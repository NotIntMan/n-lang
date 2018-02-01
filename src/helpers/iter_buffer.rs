/*!
    Итератор-буфер

    Основной целью данного модуля является уменьшение стоимости копирования итератора и повторного
    обращения к уже произведённым элементам.

    # Пример

    Чтобы не ходить вокруг, да около с научными и никому не понятными формулировками, рассмотрим пример.

    ```rust
    # use n_transpiler::helpers::iter_buffer::*;
    // Буфер может быть создан из любого объекта, для которого реализован типаж IntoIterator
    let mut buf = IterBuffer::new("a, b, y".chars());
    // Мы можем получать элементы буффера по индексу
    assert_eq!(buf.get(3), Some(&'b'));
    // А можем получить курсор и пройтись по ним итератором
    // Курсоры, к слову, обладают смещением, чтобы не усложнять логику
    let mut c = buf.cursor(1);
    assert_eq!(c.next(), Some(','));
    assert_eq!(c.next(), Some(' '));
    assert_eq!(c.next(), Some('b'));
    assert_eq!(c.next(), Some(','));
    // После итерации смещение курсора увеличивается
    assert_eq!(c.get(0), Some(&' '));
    assert_eq!(c.get(1), Some(&'y'));
    // Мы так же можем сделать курсор курсора
    let mut d = c.cursor(1);
    // И он будет со своим смещением относительно курсора-источника
    assert_eq!(d.get(0), Some(&'y'));
    ```
*/
use std::iter::{
    empty,
    Empty,
    FromIterator,
    IntoIterator,
    Iterator,
};

use lexeme_scanner::{
    Token,
    SymbolPosition,
};

use syntax_parser::basics::parser_error::{
    ParserErrorKind,
    ParserError,
};

/// Итератор-буффер
pub struct IterBuffer<T: Iterator> {
    pub source: T,
    pub buffer: Vec<T::Item>,
}

/**
    Совершенный буфер

    Вы можете использоваэ тот тип, если вам необходим функционал буфер-курсор,
    но все ваши данные уже собраны в вектор.
*/
pub type PerfectBuffer<T> = IterBuffer<Empty<T>>;

/// Тип, дающий возможность устанавливать в качестве источника курсора как буфер, так и курсор
pub enum IterBufferCursorSource<'a, T: Iterator + 'a> {
    Buffer(&'a mut IterBuffer<T>),
    Cursor(&'a mut IterBufferCursor<'a, T>),
}

/// Итератор-курсор
pub struct IterBufferCursor<'a, T: Iterator + 'a> {
    pub source: IterBufferCursorSource<'a, T>,
    pub index: usize,
}

/// Псевдоним для курсора совершенного буфера
pub type PerfectBufferCursor<'a, T> = IterBufferCursor<'a, Empty<T>>;

impl<T: Iterator> IterBuffer<T>
{
    /// Создаёт новый буфер из объекта, для которого реализован типаж IntoIterator
    pub fn new<I: IntoIterator<IntoIter = T, Item = T::Item>>(x: I) -> Self {
        IterBuffer {
            source: x.into_iter(),
            buffer: Vec::new(),
        }
    }
    /// Дополняет буфер одним элементом из источника.
    /// В случае успеха, возвращает ссылку на элемент в буфере, в противном случае возвращает None.
    pub fn fill_one(&mut self) -> Option<&T::Item> {
        let value = match self.source.next() {
            Some(v) => v,
            None => return None,
        };
        self.buffer.push(value);
        self.buffer.last()
    }
    /// Дополняет буфер несколькими элементами. Возвращает ссылку-срез на дополненные элементы.
    /// В случае полного успеха, размер среза будет равен параметру `count`.
    pub fn fill(&mut self, count: usize) -> &[T::Item] {
        let len = self.buffer.len();
        if let Some(dif) = (len + count).checked_sub(self.buffer.capacity()) {
            self.buffer.reserve(dif);
        }
        let mut filled = 0;
        'filling: for _ in 0..count {
            if self.fill_one().is_none() {
                break 'filling;
            }
            filled += 1;
        }
        &self.buffer[len..(len + filled)]
    }
    /**
        Смотрит элемент на позиции `index` и возвращает ссылку на него, если он обнаружен.
        В противном случае пытается дополнить буфер до нужного размера и, в случае успеха,
        возвращает ссылку на элемент буфера. Если источник не содержит достаточно элементов,
        возвращает None.
    */
    pub fn get(&mut self, index: usize) -> Option<&T::Item> {
        if let Some(v) = (index + 1).checked_sub(self.buffer.len()) {
            self.fill(v);
        }
        self.buffer.get(index)
    }
    /**
        Создаёт новый курсор, указывая себя в качестве источника.
        Т.к. заимствование себя является мутабельным, одновременно может
        существовать только один курсор буфера.
    */
    pub fn cursor<'a>(&'a mut self, index: usize) -> IterBufferCursor<'a, T> {
        IterBufferCursor {
            source: IterBufferCursorSource::Buffer(self),
            index,
        }
    }
    #[inline]
    pub fn peek(&mut self) -> Option<T::Item>
        where T::Item: Clone {
        match self.get(0) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}

impl<T> IterBuffer<Empty<T>> {
    /// Конструирует новый PerfectBuffer из переданного вектора
    pub fn from_vec(buffer: Vec<T>) -> PerfectBuffer<T> {
        Self {
            source: empty(),
            buffer,
        }
    }
}

impl<'a> IterBuffer<Empty<Token<'a>>> {
    pub fn get_position_of(&mut self, index: usize) -> SymbolPosition {
        if let Some(t) = self.get(index) {
            return t.pos.begin.clone();
        }
        match self.buffer.last() {
            Some(t) => t.pos.end.clone(),
            None => SymbolPosition::default(),
        }
    }
}

impl<A> FromIterator<A> for PerfectBuffer<A> {
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item=A> {
        IterBuffer::from_vec(iter.into_iter().collect())
    }
}

impl<'a, T: Iterator> IterBufferCursor<'a, T> {
    /// Вызывает метод `get` источника, передавая тому в качестве индекса `смещение+полученный индекс`
    pub fn get(&mut self, index: usize) -> Option<&T::Item> {
        let i = self.index + index;
        match &mut self.source {
            &mut IterBufferCursorSource::Buffer(ref mut buf) => buf.get(i),
            &mut IterBufferCursorSource::Cursor(ref mut cur) => cur.get(i),
        }
    }
    #[inline]
    pub fn peek(&mut self) -> Option<T::Item>
        where T::Item: Clone {
        match self.get(0) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    /**
        Создаёт новый курсор, указывая себя в качестве источника.
        Т.к. заимствование себя является мутабельным, как и у буфера,
        одновременно может существовать только один курсор курсора.
    */
    pub fn cursor(&'a mut self, index: usize) -> IterBufferCursor<'a, T> {
        IterBufferCursor {
            source: IterBufferCursorSource::Cursor(self),
            index,
        }
    }
}

/**
    Курсор так же может быть использован как итератор.

    Реализация итератора наивная, т.е. элемент, в случае его существования,
    просто клонируется. Это сделано для упрощения уравнения времени жизни элемента.
*/
impl<'a, T: Iterator> Iterator for IterBufferCursor<'a, T>
    where T::Item: Clone,
{
    type Item = T::Item;
    /// Вызывает собственный метод get с индексом 0 и, в случае успеха, клонирует результат и возвращает его
    fn next(&mut self) -> Option<Self::Item> {
        let value = match self.peek() {
            Some(v) => v.clone(),
            None => return None,
        };
        self.index += 1;
        Some(value)
    }
}

impl<'a, 'b> IterBufferCursor<'a, Empty<Token<'b>>> {
    pub fn get_position_of(&mut self, index: usize) -> SymbolPosition {
        let i = self.index + index;
        match &mut self.source {
            &mut IterBufferCursorSource::Buffer(ref mut buf) => buf.get_position_of(i),
            &mut IterBufferCursorSource::Cursor(ref mut cur) => cur.get_position_of(i),
        }
    }
    pub fn parse_error_on<T>(&mut self, index: usize, kind: ParserErrorKind) -> Result<T, ParserError> {
        Err(ParserError::from_pos(kind, self.get_position_of(index)))
    }
    pub fn get_or(&mut self, index: usize, kind: ParserErrorKind) -> Result<&Token<'b>, ParserError> {
        let pos = self.get_position_of(index);
        match self.get(index) {
            Some(v) => Ok(v),
            None => Err(ParserError::from_pos(kind, pos)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_fills_correctly() {
        let mut buf = IterBuffer::new("abc".chars());
        assert_eq!(buf.get(0), Some(&'a'));
        assert_eq!(buf.get(1), Some(&'b'));
        assert_eq!(buf.get(2), Some(&'c'));
        assert_eq!(buf.get(3), None);
    }
}
