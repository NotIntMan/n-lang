//! Набор структур для отображения позиции элемента в тексте

use std::fmt::{
    Display,
    Result,
    Formatter,
};
use std::ops::{
    Range,
    Index,
    IndexMut,
};
use std::cmp::{
    Ordering,
    PartialOrd,
};

/// Позиция символа
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolPosition {
    /**
        Означает смещение от первого символа в терминологии массивов.

        Т.е. если взять символ под номером offset из исходного текста, мы получим символ,
        на который указывает эта структура.
    */
    pub offset: usize,
    /// Означает номер строки, на которой размещён символ.
    pub line: usize,
    /**
        Означает номер столбца, на котором размещён символ.

        Строка и столбец - альтернативный вариант определения символа в тексте,
        созданный для удобства чтения человеком.
    */
    pub column: usize,
}

impl SymbolPosition {
    /// Увеличивает позицию на 1, двигая её при это вправо на 1 столбец.
    #[inline]
    pub fn step_next_column(&mut self) {
        self.offset += 1;
        self.column += 1;
    }
    /// Увеличивает позицию на 1, двигая её при это вниз, на начало следующей строки.
    #[inline]
    pub fn step_next_line(&mut self) {
        self.offset += 1;
        self.line += 1;
        self.column = 1;
    }
    /// Увеличивает позицию на 1, при решая в зависимости от переданного символа какой метод вызвать.
    /// `step_next_line` вызывается в том случае, если символ `c` является переходом на новую строку,
    /// `step_next_column` вызывается во всех прочих случаях.
    #[inline]
    pub fn step(&mut self, c: char) {
        match c {
            '\n' => self.step_next_line(),
            _ => self.step_next_column(),
        }
    }
    /// Выполняет функцию `step` для каждого символа переданной строки
    #[inline]
    pub fn step_str(&mut self, s: &str) {
        for c in s.chars() {
            self.step(c);
        }
    }
    /**
        Создаёт новый `ItemPosition` с началом в данном объекте.
        Конец `ItemPosition` вычисляется на основе данного текста.
    */
    #[inline]
    pub fn into_item_pos(self, item_text: &str) -> ItemPosition {
        let begin = self;
        let mut end = self.clone();
        end.step_str(item_text);
        ItemPosition { begin, end }
    }
    /// Выполняет клонирование данного объекта и применяет к нему метод `into_item_pos`.
    #[inline]
    pub fn make_item_pos(&self, item_text: &str) -> ItemPosition {
        self.clone().into_item_pos(item_text)
    }
}

impl Default for SymbolPosition {
    /// Значением по умоланию для данного типа является структура, указывающая на первый символ (`SymbolPosition {offset: 0, line: 1, column: 1 }`)
    fn default() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
        }
    }
}

/// Форматирование позиции, учитывающей человекочитаемость, в человекочитаемый вид - совершенно очевидное требование
impl Display for SymbolPosition {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/**
    Сравнение позиций - спорная тема

    Во первых потому, что может получиться так, что произойдёт сравнение позиций из разных систем отсчёта (разных текстов).
    В этом случае оно не имеет смысла.

    Поэтому, `partial_cmp` возвращает какой-либо результат (`Option::Some`) только в том случае,
    когда все поля `self` относятся к полям `other` соответственно одинаково.

    Это значит, что, если, например, `self.offset > other.offset`, но `self.line < other.line`,
    то происходит не корректное сравнение и метод вернёт `Option::None`.
*/
impl PartialOrd for SymbolPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let cmp_result = [
            self.offset.partial_cmp(&other.offset),
            self.line.partial_cmp(&other.line),
            self.column.partial_cmp(&other.column),
        ];
        if (cmp_result[0] == cmp_result[1]) && (cmp_result[1] == cmp_result[2]) {
            cmp_result[0]
        } else {
            None
        }
    }
}

/**
    Позиция элемента

    Служит для отображения позиции составного элемента текста. Имеет два поля типа `SymbolPosition`: начало и конец.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct ItemPosition {
    pub begin: SymbolPosition,
    pub end: SymbolPosition,
}

fn min_max(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn safe_dif(a: usize, b: usize) -> usize {
    match a.checked_sub(b) {
        Some(x) => x,
        None => b - a,
    }
}

impl ItemPosition {
    /// Формирует новый `ItemPosition` на основе текста, который был до него и текста, который включён в элемент
    pub fn new(before: &str, text: &str) -> Self {
        let mut begin = SymbolPosition::default();
        begin.step_str(before);
        let mut end = begin;
        end.step_str(text);
        ItemPosition { begin, end }
    }
    /// Возвращает длину элемента в символах
    pub fn len(&self) -> usize {
        safe_dif(self.end.offset, self.begin.offset)
    }
    /// Возвращает количество строк, на которых располагается элемент
    pub fn lines(&self) -> usize {
        1 + safe_dif(self.end.line, self.begin.line)
    }
    /// Формирует интервал (`Range`), который можно передать строке и получить текст элемента
    pub fn into_range(self) -> Range<usize> {
        let (min, max) = min_max(self.begin.offset, self.end.offset);
        min..max
    }
}

impl Index<ItemPosition> for String {
    type Output = str;
    fn index(&self, index: ItemPosition) -> &str {
        &self[index.into_range()]
    }
}

impl IndexMut<ItemPosition> for String {
    fn index_mut(&mut self, index: ItemPosition) -> &mut str {
        &mut self[index.into_range()]
    }
}
