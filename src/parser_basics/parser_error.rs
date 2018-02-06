//! Ошибка синтаксического разбора

use std::fmt::{
    Debug,
    Display,
    Result as FResult,
    Formatter,
};

use std::cmp::{
    Ord,
    Ordering,
    PartialOrd,
};

use std::mem::replace;

use lexeme_scanner::{
    TokenKindLess,
    SymbolPosition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserErrorTokenInfo {
    Kind(TokenKindLess),
    Object(TokenKindLess, String),
    Description(String),
}

/**
    Тип синтаксической ошибки.
    Самая интересная часть для того, кто собрался написать ещё пару правил.
    Тип ошибки сообщает о том, что именно произошло в процессе разбора.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserErrorKind {
    /// Неожиданный конец. Сообщает о том, что лексемы закончились, но правила этого не допускают.
    UnexpectedEnd(Option<String>),
    /// Неожиданный ввод. Сообщает о том, что ожидалась лексема одного вида, а была получена - другого.
    ExpectedGot(ParserErrorTokenInfo, ParserErrorTokenInfo),
}

/// Одиночная ошибка разбора. Применяется как элемент `ParserError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserErrorItem {
    pub kind: ParserErrorKind,
    pub pos: Option<SymbolPosition>,
}

/// Ошибка разбора. Может содержать несколько `ParserErrorItem`.
#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    One(ParserErrorItem),
    Many(Vec<ParserErrorItem>),
}

impl Display for ParserErrorTokenInfo {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            &ParserErrorTokenInfo::Kind(ref kind) => write!(f, "{}", kind),
            &ParserErrorTokenInfo::Object(ref kind, ref msg) => write!(f, "{}({})", kind, msg),
            &ParserErrorTokenInfo::Description(ref msg) => write!(f, "{}", msg),
        }
    }
}

impl ParserErrorKind {
    /// Конструирует новый `ParserErrorKind::UnexpectedEnd` с сообщением о том, что ожидался символ
    #[inline]
    pub fn unexpected_end_expected_debug<D: Debug>(c: D) -> Self {
        ParserErrorKind::UnexpectedEnd(Some(format!("{:?}", c)))
    }
    /// Конструирует новый `ParserErrorKind::UnexpectedEnd` с данным сообщением об ожидании
    #[inline]
    pub fn unexpected_end_expected<S: ToString>(msg: S) -> Self {
        ParserErrorKind::UnexpectedEnd(Some(msg.to_string()))
    }
    /// Конструирует новый `ParserErrorKind::UnexpectedEnd` без сообщения
    #[inline]
    pub fn unexpected_end() -> Self {
        ParserErrorKind::UnexpectedEnd(None)
    }
    #[inline]
    pub fn expected_got_kind(expected: TokenKindLess, got: TokenKindLess) -> Self {
        let a = ParserErrorTokenInfo::Kind(expected);
        let b = ParserErrorTokenInfo::Kind(got);
        ParserErrorKind::ExpectedGot(a, b)
    }
    #[inline]
    pub fn expected_got_kind_text<A: ToString, B: ToString>(expected_kind: TokenKindLess, expected_text: A, got_kind: TokenKindLess, got_text: B) -> Self {
        let a = ParserErrorTokenInfo::Object(expected_kind, expected_text.to_string());
        let b = ParserErrorTokenInfo::Object(got_kind, got_text.to_string());
        ParserErrorKind::ExpectedGot(a, b)
    }
    #[inline]
    pub fn expected_got_description<A: ToString, B: ToString>(expected: A, got_kind: TokenKindLess, got_text: B) -> Self {
        let a = ParserErrorTokenInfo::Description(expected.to_string());
        let b = ParserErrorTokenInfo::Object(got_kind, got_text.to_string());
        ParserErrorKind::ExpectedGot(a, b)
    }
}

/// Типаж Display у `ParserErrorKind` служит для отображения типа ошибки в человекочитаемом виде
impl Display for ParserErrorKind {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            &ParserErrorKind::UnexpectedEnd(ref s) => {
                write!(f, "unexpected end")?;
                if let &Some(ref m) = s {
                    write!(f, ", expected: {}", m)?;
                }
                Ok(())
            },
            &ParserErrorKind::ExpectedGot(ref exp, ref got) => write!(f, "expected: {}, got: {}", exp, got),
        }
    }
}

impl ParserErrorItem {
    /// Конструирует новую единицу ошибки из типа и позиции
    #[inline]
    const fn new(kind: ParserErrorKind, pos: SymbolPosition) -> Self {
        Self {
            kind,
            pos: Some(pos),
        }
    }
    /// Конструирует новую единицу ошибки из типа, но без позиции
    #[inline]
    const fn new_without_pos(kind: ParserErrorKind) -> Self {
        Self {
            kind,
            pos: None,
        }
    }
}

/// Типаж Display у `ParserErrorItem` служит для отображения ошибки в человекочитаемом виде
impl Display for ParserErrorItem {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "{}", self.kind)?;
        if let &Some(ref pos) = &self.pos {
            write!(f, " on {}", pos)?;
        }
        Ok(())
    }
}

impl PartialOrd for ParserErrorItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.pos.partial_cmp(&other.pos)
    }
}

impl Ord for ParserErrorItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("Trying to sort error from different modules")
    }
}

impl ParserError {
    /// Конструирует единичную ошибку из типа и позиции
    #[inline]
    pub const fn new(kind: ParserErrorKind, pos: SymbolPosition) -> ParserError {
        ParserError::One(
            ParserErrorItem::new(kind, pos)
        )
    }
    /// Конструирует единичную ошибку из типа, но без позиции
    #[inline]
    pub const fn new_without_pos(kind: ParserErrorKind) -> ParserError {
        ParserError::One(
            ParserErrorItem::new_without_pos(kind)
        )
    }
    /// Выполняет копирование всех хранимых ошибок в вектор и возвращает его
    #[inline]
    pub fn extract_into_vec(&self) -> Vec<ParserErrorItem> {
        match self {
            &ParserError::One(ref e) => vec![e.clone()],
            &ParserError::Many(ref v) => v.clone(),
        }
    }
    /// Выполняет поглощение другой ошибки.
    /// После выполнения текущий объект будет содержать как свои элементы, так и элементы из переданного объекта.
    pub fn append(&mut self, err: ParserError) {
        let result = match self {
            &mut ParserError::One(ref e) => {
                let e = e.clone();
                let v = match err {
                    ParserError::One(f) => {
                        vec![e, f]
                    },
                    ParserError::Many(mut w) => {
                        w.push(e);
                        w
                    },
                };
                ParserError::Many(v)
            },
            &mut ParserError::Many(ref mut v) => {
                match err {
                    ParserError::One(e) => v.push(e),
                    ParserError::Many(mut w) => v.append(&mut w),
                }
                return;
            },
        };
        replace(self, result);
    }
}

/// Типаж Display у `ParserError` служит для отображения группы ошибок в человекочитаемом виде
impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        let mut errors = self.extract_into_vec();
        errors.sort();
        writeln!(f, "There are some errors:")?;
        for (i, error) in errors.into_iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, error)?;
        }
        writeln!(f, "Solution of one of them may solve the problem.")
    }
}
