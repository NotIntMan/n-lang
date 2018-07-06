use helpers::{
    Path,
    PathBuf,
};
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
use std::fmt;

#[derive(Debug, Clone, Eq, Hash)]
pub struct ItemPath {
    pub pos: ItemPosition,
    pub path: PathBuf,
}

impl ItemPath {
    #[inline]
    pub fn for_root(delimiter: &str) -> Self {
        ItemPath {
            pos: Default::default(),
            path: PathBuf::from(Path::new("", delimiter)),
        }
    }
    pub fn new(pos: ItemPosition, idents: Vec<Identifier>, delimiter: &str) -> Self {
        let mut path = PathBuf::new(delimiter);
        for ident in idents {
            path.push(ident.text());
        }
        ItemPath {
            pos,
            path,
        }
    }
}

impl PartialEq for ItemPath {
    #[inline]
    fn eq(&self, other: &ItemPath) -> bool {
        (self.pos == other.pos) &&
            (self.path == other.path)
    }

    #[inline]
    fn ne(&self, other: &ItemPath) -> bool {
        (self.pos != other.pos) |
            (self.path != other.path)
    }
}

/// Реализует разбор "пути" элементов, разделённых делителем. Отличается от списка тем, что не позволяет "замыкающий делитель".
pub fn path<'token, 'source>(input: &'token [Token<'source>], delimiter: &'source str) -> ParserResult<'token, 'source, ItemPath> {
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
            ItemPath::new(pos, path, delimiter)
        })
    )
}

/// Реализует разбор "пути свойства" (напр., "foo.bar.baz")
pub fn property_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, ItemPath> {
    path(input, ".")
}

/// Реализует разбор "пути модуля" (напр., "foo::bar::baz")
pub fn module_path<'token, 'source>(input: &'token [Token<'source>]) -> ParserResult<'token, 'source, ItemPath> {
    path(input, "::")
}

pub fn write_path<W: fmt::Write>(w: &mut W, path: &[Identifier], delimiter: &str) -> fmt::Result {
    let mut path_iter = path.iter();
    if let Some(path_item) = path_iter.next() {
        write!(w, "{}", path_item.text())?;
    }
    for path_item in path_iter {
        write!(w, "{}{}", delimiter, path_item.text())?;
    }
    Ok(())
}
