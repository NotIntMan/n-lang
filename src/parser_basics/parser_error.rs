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

use helpers::group::{
    Appendable,
    Group,
};
use helpers::display_list::display_list;

use lexeme_scanner::{
    TokenKindLess,
    SymbolPosition,
};

/**
    Тип, отображающий некоторый объект текста.

    Существует только для того, чтобы помочь варианту `ParserErrorKind::ExpectedGot` не размножиться
    на 8 штук только из-за необходимости вариативности отображения объектов.
*/
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParserErrorTokenInfo {
    /// Поле `kind` отображает тип токена
    pub kind: Option<TokenKindLess>,
    /// Поле `desc` отображает описание токена
    pub desc: Option<String>,
}

impl ParserErrorTokenInfo {
    /// Создаёт новый объект информации, честно заполняя поля в соответствии переданным аргументам
    #[inline]
    pub fn new(kind: Option<TokenKindLess>, desc: Option<String>) -> Self {
        Self { kind, desc }
    }
    #[inline]
    /// Создаёт новый объект информации, заполняя значением только поле `kind`
    pub fn from_kind(kind: TokenKindLess) -> Self {
        ParserErrorTokenInfo::new(Some(kind), None)
    }
    #[inline]
    /// Создаёт новый объект информации, заполняя значением только поле `desc`
    pub fn from_desc(desc: String) -> Self {
        ParserErrorTokenInfo::new(None, Some(desc))
    }
    /// Выполняет "групппировку ожиданий" - второй вариант группировки, ожидаемый от этой структуры.
    /// Не вынесен в отдельную структуру из-за ненадобности, т.к. используется только в `ParserErrorKind`.
    pub fn append_expectation(&mut self, other: Self) -> Option<Self> {
        if self.kind != other.kind {
            return Some(other);
        }
        'out: loop {
            self.desc = match &self.desc {
                &Some(ref desc) => {
                    if let &Some(ref other_desc) = &other.desc {
                        if desc != other_desc {
                            break 'out;
                        }
                    }
                    return None;
                }
                &None => other.desc,
            };
            return None;
        }
        return Some(other);
    }
}

impl Display for ParserErrorTokenInfo {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            &ParserErrorTokenInfo { kind: Some(ref kind), desc: None } => write!(f, "{}", kind),
            &ParserErrorTokenInfo { kind: Some(ref kind), desc: Some(ref msg) } => write!(f, "{} {:?}", kind, msg),
            &ParserErrorTokenInfo { kind: None, desc: Some(ref msg) } => write!(f, "{}", msg),
            _ => Ok(()),
        }
    }
}

impl Appendable for ParserErrorTokenInfo {}

/**
    Тип синтаксической ошибки.
    Самая интересная часть для того, кто собрался написать ещё пару правил.
    Тип ошибки сообщает о том, что именно произошло в процессе разбора.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserErrorKind {
    /// Неожиданный конец. Сообщает о том, что лексемы закончились, но правила этого не допускают.
    UnexpectedEnd(Group<String>),
    /// Неожиданный ввод. Сообщает о том, что ожидалась лексема одного вида, а была получена - другого.
    ExpectedGot(Group<ParserErrorTokenInfo>, ParserErrorTokenInfo),
    /// Ключ не уникален. Сообщает о том, что в определении структуры находится два поля с одинаковым именем.
    KeyIsNotUnique(Group<String>),
    /// Прочая ошибка. Сообщает о том, что произошло что-то где-то за пределами парсера.
    CustomError(Group<String>),
}

impl ParserErrorKind {
    /// Конструирует новый `ParserErrorKind::UnexpectedEnd` с сообщением о том, что ожидался символ
    #[inline]
    pub fn unexpected_end_expected_debug<D: Debug>(c: D) -> Self {
        ParserErrorKind::UnexpectedEnd(Group::One(format!("{:?}", c)))
    }
    /// Конструирует новый `ParserErrorKind::UnexpectedEnd` с данным сообщением об ожидании
    #[inline]
    pub fn unexpected_end_expected<S: ToString>(msg: S) -> Self {
        ParserErrorKind::UnexpectedEnd(Group::One(msg.to_string()))
    }
    /// Конструирует новый `ParserErrorKind::UnexpectedEnd` без сообщения
    #[inline]
    pub fn unexpected_end() -> Self {
        ParserErrorKind::UnexpectedEnd(Group::None)
    }
    /// Конструирует новый `ParserErrorKind::ExpectedGot`, содержащий инофрмацию о типе ожидаемого и полученного токенов
    #[inline]
    pub fn expected_got_kind(expected: TokenKindLess, got: TokenKindLess) -> Self {
        let a = Group::One(ParserErrorTokenInfo::from_kind(expected));
        let b = ParserErrorTokenInfo::from_kind(got);
        ParserErrorKind::ExpectedGot(a, b)
    }
    /// Конструирует новый `ParserErrorKind::ExpectedGot`, содержащий инофрмацию о типе и тексте ожидаемого и полученного токенов
    #[inline]
    pub fn expected_got_kind_text<A: ToString, B: ToString>(expected_kind: TokenKindLess, expected_text: A, got_kind: TokenKindLess, got_text: B) -> Self {
        let a = Group::One(ParserErrorTokenInfo::new(Some(expected_kind), Some(expected_text.to_string())));
        let b = ParserErrorTokenInfo::new(Some(got_kind), Some(got_text.to_string()));
        ParserErrorKind::ExpectedGot(a, b)
    }
    /// Конструирует новый `ParserErrorKind::ExpectedGot`, содержащий описание ожидаемого токена и инофрмацию о типе и тексте полученного токена
    #[inline]
    pub fn expected_got_description<A: ToString, B: ToString>(expected: A, got_kind: TokenKindLess, got_text: B) -> Self {
        let a = Group::One(ParserErrorTokenInfo::from_desc(expected.to_string()));
        let b = ParserErrorTokenInfo::new(Some(got_kind), Some(got_text.to_string()));
        ParserErrorKind::ExpectedGot(a, b)
    }
    /// Конструирует новый `ParserErrorKind::NomError`, содержащий сообщение об ошибке комбинатора парсеров
    #[inline]
    pub fn custom_error<A: ToString>(msg: A) -> Self {
        ParserErrorKind::CustomError(Group::One(msg.to_string()))
    }
    /// Конструирует новый `ParserErrorKind::KeyIsNotUnique`, содержащий сообщение имя повторяющегося ключа
    #[inline]
    pub fn key_is_not_unique<A: ToString>(msg: A) -> Self {
        ParserErrorKind::KeyIsNotUnique(Group::One(msg.to_string()))
    }
}

impl Appendable for ParserErrorKind {
    fn append(&mut self, other: Self) -> Option<Self> {
        if *self == other {
            return None;
        }
        match self {
            &mut ParserErrorKind::UnexpectedEnd(ref mut self_group) => {
                match other {
                    ParserErrorKind::UnexpectedEnd(other_group) => {
                        self_group.append_group(other_group);
                        None
                    }
                    other_else => {
                        Some(other_else)
                    }
                }
            }
            &mut ParserErrorKind::ExpectedGot(ref mut self_group, ref mut self_info) => {
                match other {
                    ParserErrorKind::ExpectedGot(other_group, other_info) => {
                        match self_info.append_expectation(other_info) {
                            Some(other_info) => {
                                Some(ParserErrorKind::ExpectedGot(other_group, other_info))
                            }
                            None => {
                                self_group.append_group(other_group);
                                None
                            }
                        }
                    }
                    other_else => {
                        Some(other_else)
                    }
                }
            }
            &mut ParserErrorKind::KeyIsNotUnique(ref mut self_group) => {
                match other {
                    ParserErrorKind::KeyIsNotUnique(other_group) => {
                        self_group.append_group(other_group);
                        None
                    }
                    other_else => Some(other_else),
                }
            }
            &mut ParserErrorKind::CustomError(ref mut self_group) => {
                match other {
                    ParserErrorKind::CustomError(other_group) => {
                        self_group.append_group(other_group);
                        None
                    }
                    other_else => Some(other_else),
                }
            }
        }
    }
}

/// Типаж Display у `ParserErrorKind` служит для отображения типа ошибки в человекочитаемом виде
impl Display for ParserErrorKind {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            &ParserErrorKind::UnexpectedEnd(ref s) => {
                write!(f, "unexpected end")?;
                let expectations = s.extract_into_vec();
                if expectations.len() > 0 {
                    write!(f, ", expected: ")?;
                    display_list(f, &expectations)?;
                }
                Ok(())
            }
            &ParserErrorKind::ExpectedGot(ref exp, ref got) => {
                write!(f, "expected: ")?;
                display_list(f, &exp.extract_into_vec())?;
                write!(f, ", got: {}", got)?;
                Ok(())
            }
            &ParserErrorKind::KeyIsNotUnique(ref key) => {
                write!(f, "key")?;
                display_list(f, &key.extract_into_vec())?;
                write!(f, "is not unique")?;
                Ok(())
            }
            &ParserErrorKind::CustomError(ref messages) => display_list(f, &messages.extract_into_vec()),
        }
    }
}

impl Default for ParserErrorKind {
    /// Нейтральным значением `ParserErrorKind` является `ParserErrorKind::CustomError(Group::None)`
    fn default() -> Self {
        ParserErrorKind::CustomError(Group::None)
    }
}

/// Одиночная ошибка разбора. Применяется как элемент `ParserError`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParserErrorItem {
    pub kind: ParserErrorKind,
    pub pos: Option<SymbolPosition>,
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

impl Appendable for ParserErrorItem {
    fn append(&mut self, other: Self) -> Option<Self> {
        if *self == other {
            return None;
        }
        if self.pos != other.pos {
            return Some(other);
        }
        let ParserErrorItem {
            kind: other_kind,
            pos: other_pos,
        } = other;
        match self.kind.append(other_kind) {
            Some(other_kind) => {
                Some(ParserErrorItem {
                    kind: other_kind,
                    pos: other_pos,
                })
            }
            None => {
                None
            }
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

/// Ошибка разбора. Может содержать несколько `ParserErrorItem`.
pub type ParserError = Group<ParserErrorItem>;

impl Group<ParserErrorItem> {
    /// Конструирует единичную ошибку из типа и позиции
    #[inline]
    pub const fn new(kind: ParserErrorKind, pos: SymbolPosition) -> ParserError {
        Group::One(
            ParserErrorItem::new(kind, pos)
        )
    }
    /// Конструирует единичную ошибку из типа, но без позиции
    #[inline]
    pub const fn new_without_pos(kind: ParserErrorKind) -> ParserError {
        Group::One(
            ParserErrorItem::new_without_pos(kind)
        )
    }
}

/// Типаж Display у `ParserError` служит для отображения группы ошибок в человекочитаемом виде
impl Display for Group<ParserErrorItem> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            &Group::None => write!(f, "There are no errors detected."),
            &Group::One(ref err) => write!(f, "There is error: {}", err),
            &Group::Many(ref vec) => {
                let mut errors = vec.clone();
                errors.sort();
                writeln!(f, "There are some errors:")?;
                for (i, error) in errors.into_iter().enumerate() {
                    writeln!(f, "  {}. {}", i + 1, error)?;
                }
                writeln!(f, "Solution of one of them may solve the problem.")
            },
        }
    }
}
