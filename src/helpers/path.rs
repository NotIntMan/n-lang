use std::{
    cmp,
    fmt::{
        self,
        Write,
    },
    mem::replace,
};

#[derive(Clone, Eq, Hash)]
pub struct PathBuf {
    pub data: String,
    pub delimiter: String,
}

impl PathBuf {
    #[inline]
    pub fn empty() -> Self {
        PathBuf::new("")
    }
    pub fn new(delimiter: &str) -> Self {
        PathBuf {
            data: String::new(),
            delimiter: delimiter.to_string(),
        }
    }
    pub fn from_path(path: Path) -> Self {
        let data = path.data.to_string();
        let delimiter = path.delimiter.to_string();
        PathBuf {
            data,
            delimiter,
        }
    }
    pub fn from_paths(first: Path, second: Path) -> Self {
        let mut buf = PathBuf::from_path(first);
        buf.append(second);
        buf
    }
    #[inline]
    pub fn as_path(&self) -> Path {
        Path {
            data: &self.data,
            delimiter: &self.delimiter,
        }
    }
    pub fn append(&mut self, additional: Path) {
        self.data.reserve_exact(additional.data.len());
        for component in additional {
            self.push(component);
        }
    }
    pub fn push(&mut self, component: &str) {
        self.data.reserve_exact(component.len() + self.delimiter.len());
        if !self.data.is_empty() {
            self.data.push_str(&self.delimiter);
        }
        self.data.push_str(component);
    }
    pub fn push_fmt(&mut self, component: impl fmt::Display) -> fmt::Result {
        self.data.reserve_exact(16 + self.delimiter.len());
        if !self.data.is_empty() {
            self.data.push_str(&self.delimiter);
        }
        self.data.write_fmt(format_args!("{}", component))
    }
    pub fn push_front(&mut self, component: impl ToString) {
        self.data.reserve_exact(16 + self.delimiter.len());
        if !self.data.is_empty() {
            self.data.replace_range(0..0, &self.delimiter);
        }
        self.data.replace_range(0..0, &component.to_string());
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_path().is_empty()
    }
    #[inline]
    pub fn the_only(&self) -> Option<&str> {
        self.as_path().the_only()
    }
}

impl fmt::Debug for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PathBuf: {:?}", self.data)
    }
}

impl fmt::Display for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl cmp::PartialEq for PathBuf {
    fn eq(&self, other: &PathBuf) -> bool {
        self.as_path() == other.as_path()
    }
    fn ne(&self, other: &PathBuf) -> bool {
        self.as_path() != other.as_path()
    }
}

impl<'a> cmp::PartialEq<Path<'a>> for PathBuf {
    fn eq(&self, other: &Path) -> bool {
        self.as_path() == *other
    }
    fn ne(&self, other: &Path) -> bool {
        self.as_path() != *other
    }
}

impl<'a> cmp::PartialEq<PathBuf> for Path<'a> {
    fn eq(&self, other: &PathBuf) -> bool {
        *self == other.as_path()
    }
    fn ne(&self, other: &PathBuf) -> bool {
        *self != other.as_path()
    }
}

impl cmp::PartialOrd for PathBuf {
    fn partial_cmp(&self, other: &PathBuf) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for PathBuf {
    fn cmp(&self, other: &PathBuf) -> cmp::Ordering {
        match self.data.cmp(&other.data) {
            cmp::Ordering::Equal => {
                self.delimiter.cmp(&other.delimiter)
            }
            o => o,
        }
    }
}

#[derive(Clone, Copy, Eq, Hash)]
pub struct Path<'a> {
    pub data: &'a str,
    pub delimiter: &'a str,
}

impl<'a> Path<'a> {
    #[inline]
    pub fn empty() -> Self {
        Path::new("", "")
    }
    #[inline]
    pub fn new(data: &'a str, delimiter: &'a str) -> Self {
        Path {
            data,
            delimiter,
        }
    }
    #[inline]
    pub fn components(self) -> PathComponents<'a> {
        let Path { data, delimiter } = self;
        PathComponents {
            data,
            delimiter,
        }
    }
    pub fn is_begin_of(self, other: Path) -> Option<Path> {
        let mut other_components = other.components();
        for self_component in self.components() {
            match other_components.next() {
                Some(other_component) => if self_component != other_component {
                    return None;
                }
                None => return None,
            }
        }
        Some(other_components.into_path())
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        *self == ([] as [&str; 0])[..]
    }
    pub fn pop_left(&mut self) -> Option<&'a str> {
        let mut components = self.components();
        let result = components.next();
        replace(self, components.into_path());
        result
    }
    pub fn pop_right(&mut self) -> Option<&'a str> {
        let delimiter_length = self.delimiter.len();
        let mut shrink_point = match self.data.len().checked_sub(delimiter_length) {
            Some(x) => x,
            None => 0,
        };
        loop {
            if shrink_point == 0 {
                let result = self.data;
                self.data = "";
                return Some(result);
            }
            let last_begin = shrink_point + delimiter_length;
            let may_be_delimiter = &self.data[shrink_point..last_begin];
            if may_be_delimiter == self.delimiter {
                let result = &self.data[last_begin..];
                self.data = &self.data[..shrink_point];
                return Some(result);
            }
            shrink_point -= 1;
        }
    }
    pub fn the_only(self) -> Option<&'a str> {
        let mut components = self.components();
        let result = components.next()?;
        if components.next().is_none() {
            Some(result)
        } else {
            None
        }
    }
    #[inline]
    pub fn into_buf(self) -> PathBuf {
        PathBuf::from_path(self)
    }
    #[inline]
    pub fn into_new_buf(self, delimiter: &str) -> PathBuf {
        let mut result = PathBuf::new(delimiter);
        for item in self {
            result.push(item);
        }
        result
    }
}

impl<'a> fmt::Debug for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Path: {:?}", self.data)
    }
}

impl<'a> fmt::Display for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl<'a> IntoIterator for Path<'a> {
    type Item = &'a str;
    type IntoIter = PathComponents<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.components()
    }
}

impl<'a> cmp::PartialEq for Path<'a> {
    fn eq(&self, other: &Path) -> bool {
        self.components().eq(other.components())
    }
    fn ne(&self, other: &Path) -> bool {
        self.components().ne(other.components())
    }
}

impl<'a> cmp::PartialEq<[&'a str]> for Path<'a> {
    fn eq(&self, other: &[&'a str]) -> bool {
        self.components().eq(other.into_iter().cloned())
    }
    fn ne(&self, other: &[&'a str]) -> bool {
        self.components().ne(other.into_iter().cloned())
    }
}

impl<'a> Into<PathBuf> for Path<'a> {
    fn into(self) -> PathBuf {
        self.into_buf()
    }
}

impl<'a> cmp::PartialOrd for Path<'a> {
    fn partial_cmp(&self, other: &Path<'a>) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> cmp::Ord for Path<'a> {
    fn cmp(&self, other: &Path<'a>) -> cmp::Ordering {
        match self.data.cmp(&other.data) {
            cmp::Ordering::Equal => {
                self.delimiter.cmp(&other.delimiter)
            }
            o => o,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PathComponents<'a> {
    data: &'a str,
    delimiter: &'a str,
}

impl<'a> PathComponents<'a> {
    pub fn into_path(self) -> Path<'a> {
        let PathComponents { data, delimiter } = self;
        Path { data, delimiter }
    }
}

impl<'a> Iterator for PathComponents<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let data_length = self.data.len();
        if data_length == 0 {
            return None;
        }
        let delimiter_length = self.delimiter.len();
        let mut component_end = 0;
        let mut delimiter_end;
        loop {
            delimiter_end = component_end + delimiter_length;
            if delimiter_end >= data_length {
                component_end = data_length;
                delimiter_end = data_length;
                break;
            }
            if &self.data[component_end..delimiter_end] == self.delimiter {
                break;
            }
            component_end += 1;
        }
        let result = &self.data[..component_end];
        let new_data = &self.data[delimiter_end..];
        self.data = new_data;
        Some(result)
    }
}

impl<'a> Into<Path<'a>> for PathComponents<'a> {
    fn into(self) -> Path<'a> {
        self.into_path()
    }
}

#[test]
fn a() {
    let p = Path::new("foo::bar", "::");
    let mut components = p.components();
    assert_eq!(components.next(), Some("foo"));
    assert_eq!(components.next(), Some("bar"));
    assert_eq!(components.next(), None);
}

#[test]
fn b() {
    let p0 = Path::new("foo::bar", "::");
    let p1 = Path::new("click->for", "->");
    let buf = PathBuf::from_paths(p0, p1);
    assert_eq!(buf.data, "foo::bar::click::for");
}

#[test]
fn c() {
    assert!(Path::empty().is_empty());
    assert!(PathBuf::empty().as_path().is_empty());
}
