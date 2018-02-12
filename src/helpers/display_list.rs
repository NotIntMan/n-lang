use std::fmt::{
    Display,
    Result as FResult,
    Formatter,
};

/// Функция-помощник, которая форматирует список значений, разделяя их запятой
pub fn display_list<T: Display>(formatter: &mut Formatter, source: &[T]) -> FResult {
    let mut iter = source.iter();
    if let Some(item) = iter.next() {
        write!(formatter, "{}", item)?;
    }
    for item in iter {
        write!(formatter, ", {}", item)?;
    }
    Ok(())
}

/// Структура, оборачивающая функциюнал функции `display_list` для использования в привычном форматировании
pub struct DisplayList<'a, T: 'a>(&'a [T]);

impl<'a, T: 'a + Display> Display for DisplayList<'a, T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult {
        display_list(f, self.0)
    }
}
