use helpers::{
    BlockFormatter,
    CodeFormatter,
    Path,
};
use std::fmt;

pub trait Format<T> {
    fn fmt(&self, f: &mut impl fmt::Write, parameters: T) -> fmt::Result;
}

pub trait Generate<T> {
    fn fmt(&self, f: BlockFormatter<impl fmt::Write>, parameters: T) -> fmt::Result;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TSQLParameters<'a> {
    pub module_path: Path<'a>,
    pub indent_size: usize,
}

impl<'a> TSQLParameters<'a> {
    pub fn new(module_path: Path<'a>) -> Self {
        Self {
            module_path,
            indent_size: 2,
        }
    }
    pub fn with_indent_size(self, indent_size: usize) -> Self {
        Self {
            indent_size,
            ..self
        }
    }
}

impl<'a, T: Generate<TSQLParameters<'a>>> Format<TSQLParameters<'a>> for T {
    fn fmt(&self, f: &mut impl fmt::Write, parameters: TSQLParameters<'a>) -> fmt::Result {
        let mut formatter = CodeFormatter::new(f);
        formatter.indent_size = parameters.indent_size;
        Generate::fmt(self, formatter.root_block(), parameters)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TSQL<'a, 'b, T: 'a + Format<TSQLParameters<'b>>>(pub &'a T, pub TSQLParameters<'b>);

impl<'a, 'b, T: 'a + Format<TSQLParameters<'b>>> fmt::Display for TSQL<'a, 'b, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f, self.1.clone())
    }
}
