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
    TokenKind,
    SymbolPosition,
};

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
    ExpectedGotKind(TokenKind, TokenKind),
    /// Другая разновидность ошибки "Неожиданный ввод".
    /// Сообщает о том, что ожидалась лексема одного <i>типа и содержания</i>, а получена - другого.
    ExpectedGotKindText((TokenKind, String), (TokenKind, String)),
    /// Ещё одна разновидность ошибки "Неожиданный ввод".
    /// Сообщает о том, что ожидалось нечто специфическое, но было получено нечто другое.
    ExpectedGotMessage(String, (TokenKind, String)),
}

/// Одиночная ошибка разбора. Применяется как элемент `ParserError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserErrorItem {
    pub kind: ParserErrorKind,
    pub pos: SymbolPosition,
}

/// Ошибка разбора. Может содержать несколько `ParserErrorItem`.
#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    One(ParserErrorItem),
    Many(Vec<ParserErrorItem>),
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
            &ParserErrorKind::ExpectedGotKind(ref exp, ref got) => write!(f, "expected: {:?}, got: {:?}", exp, got),
            &ParserErrorKind::ExpectedGotKindText(
                (ref exp_kind, ref exp_text), (ref got_kind, ref got_text)
            ) => write!(f, "expected: {:?}({:?}), got: {:?}({:?})", exp_kind, exp_text, got_kind, got_text),
            &ParserErrorKind::ExpectedGotMessage(ref exp, (ref got_kind, ref got_text)) => write!(f, "expected: {}, got: {:?}({:?})", exp, got_kind, got_text),
        }
    }
}

impl ParserErrorItem {
    /// Конструирует новую единицу ошибки из типа и позиции
    #[inline]
    const fn from_pos(kind: ParserErrorKind, pos: SymbolPosition) -> Self {
        Self {
            kind,
            pos,
        }
    }
}

/// Типаж Display у `ParserErrorItem` служит для отображения ошибки в человекочитаемом виде
impl Display for ParserErrorItem {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "{} on {}", self.kind, self.pos)
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
    pub const fn from_pos(kind: ParserErrorKind, pos: SymbolPosition) -> ParserError {
        ParserError::One(
            ParserErrorItem::from_pos(kind, pos)
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
