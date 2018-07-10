use std::fmt;

pub trait Format<T> {
    fn fmt(&self, f: &mut impl fmt::Write) -> fmt::Result;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TSQLParameters;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TSQL<T: Format<TSQLParameters>>(pub T);

impl<T: Format<TSQLParameters>> fmt::Display for TSQL<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
