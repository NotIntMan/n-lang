use std::fmt;
use helpers::into_static::IntoStatic;
use lexeme_scanner::{
    ItemPosition,
    Token,
};
use parser_basics::{
    identifier,
    Identifier,
    item_position,
    ParserResult,
    symbol_position,
    symbols,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path<'source> {
    pub pos: ItemPosition,
    pub path: Vec<Identifier<'source>>,
}

pub type StaticPath = Path<'static>;

impl<'source> IntoStatic for Path<'source> {
    type Result = StaticPath;
    fn into_static(self) -> StaticPath {
        let Path { pos, path } = self;
        Path {
            pos,
            path: path.into_static(),
        }
    }
}

/// Реализует разбор "пути" элементов, разделённых делителем. Отличается от списка тем, что не позволяет "замыкающий делитель".
pub fn path<'token, 'source>(input: &'token [Token<'source>], delimiter: &'source str) -> ParserResult<'token, 'source, Path<'source>> {
    do_parse!(input,
        begin: symbol_position >>
        first: identifier >>
        others: many0!(do_parse!(
            apply!(symbols, delimiter) >>
            element: identifier >>
            (element)
        )) >>
        pos: apply!(item_position, begin) >>
        ({
            let mut path = others;
            path.insert(0, first);
            #[cfg(feature="parser_trace")]
            trace!("Path found: {:?}", path);
            Path { path, pos, }
        })
    )
}

/// Реализует разбор "пути свойства" (напр., "foo.bar.baz")
pub fn property_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Path<'source>> {
    path(input, ".")
}

/// Реализует разбор "пути модуля" (напр., "foo::bar::baz")
pub fn module_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, Path<'source>> {
    path(input, "::")
}

pub fn write_path<W: fmt::Write>(w: &mut W, path: &[Identifier], delimiter: &str) -> fmt::Result {
    let mut path_iter = path.iter();
    if let Some(path_item) = path_iter.next() {
        write!(w, "{}", path_item.get_text())?;
    }
    for path_item in path_iter {
        write!(w, "{}{}", delimiter, path_item.get_text())?;
    }
    Ok(())
}
