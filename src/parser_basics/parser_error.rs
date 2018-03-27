//! Ошибка синтаксического разбора

use std::fmt::{
    Display,
    Result as FResult,
    Formatter,
};
use std::cmp::{
    Ord,
    Ordering,
    PartialOrd,
};
use std::borrow::Cow;
use helpers::group::{
    Appendable,
    Group,
};
use helpers::display_list::display_list;
use helpers::into_static::IntoStatic;
use lexeme_scanner::{
    TokenKindLess,
    SymbolPosition,
};

/**
    Тип, отображающий некоторый объект текста.

    Существует только для того, чтобы помочь варианту `ParserErrorKind::ExpectedGot` не размножиться
    на 8 штук только из-за необходимости вариативности отображения объектов.
*/
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct ParserErrorTokenInfo<'source> {
    /// Поле `kind` отображает тип токена
    pub kind: Option<TokenKindLess>,
    /// Поле `desc` отображает описание токена
    pub desc: Option<Cow<'source, str>>,
}

impl<'source> ParserErrorTokenInfo<'source> {
    /// Создаёт новый объект информации, честно заполняя поля в соответствии переданным аргументам
    #[inline]
    pub fn new(kind: Option<TokenKindLess>, desc: Option<&'source str>) -> Self {
        Self { kind, desc: desc.map(Cow::Borrowed) }
    }
    #[inline]
    /// Создаёт новый объект информации, заполняя значением только поле `kind`
    pub fn from_kind(kind: TokenKindLess) -> Self {
        ParserErrorTokenInfo::new(Some(kind), None)
    }
    #[inline]
    /// Создаёт новый объект информации, заполняя значением только поле `desc`
    pub fn from_desc(desc: &'source str) -> Self {
        ParserErrorTokenInfo::new(None, Some(desc))
    }
    #[inline]
    /// Создаёт новый объект информации, заполняя значением поля `kind` и `desc`
    pub fn from_kind_and_desc(kind: TokenKindLess, desc: &'source str) -> Self {
        ParserErrorTokenInfo::new(Some(kind), Some(desc))
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

impl<'source> Display for ParserErrorTokenInfo<'source> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        match self {
            &ParserErrorTokenInfo { kind: Some(ref kind), desc: None } => write!(f, "{}", kind),
            &ParserErrorTokenInfo { kind: Some(ref kind), desc: Some(ref msg) } => {
                write!(f, "{}", kind)?;
                if !msg.is_empty() {
                    write!(f, " {:?}", msg)?;
                }
                Ok(())
            },
            &ParserErrorTokenInfo { kind: None, desc: Some(ref msg) } => write!(f, "{}", msg),
            _ => Ok(()),
        }
    }
}

impl<'source> Appendable for ParserErrorTokenInfo<'source> {
    fn append(&mut self, other: Self) -> Option<Self> {
        if *self == other { None } else { Some(other) }
    }
}

impl<'source> IntoStatic for ParserErrorTokenInfo<'source> {
    type Result = ParserErrorTokenInfo<'static>;
    fn into_static(self) -> Self::Result {
        let ParserErrorTokenInfo { kind, desc } = self;
        ParserErrorTokenInfo {
            kind,
            desc: desc.map(|desc| Cow::Owned(desc.into_owned())),
        }
    }
}

/**
    Тип синтаксической ошибки.
    Самая интересная часть для того, кто собрался написать ещё пару правил.
    Тип ошибки сообщает о том, что именно произошло в процессе разбора.
*/
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParserErrorKind<'source> {
    /// Неожиданный конец. Сообщает о том, что лексемы закончились, но правила этого не допускают.
    UnexpectedEnd(Group<ParserErrorTokenInfo<'source>>),
    /// Неожиданный ввод. Сообщает о том, что ожидалась лексема одного вида, а была получена - другого.
    ExpectedGot(Group<ParserErrorTokenInfo<'source>>, ParserErrorTokenInfo<'source>),
    /// Прочая ошибка. Сообщает о том, что произошло что-то где-то за пределами парсера.
    CustomError(Group<String>),
}

impl<'source> ParserErrorKind<'source> {
    /// Конструирует новый `ParserErrorKind::UnexpectedEnd` с данным сообщением об ожидании
    #[inline]
    pub fn unexpected_end_expected(idem: ParserErrorTokenInfo<'source>) -> Self {
        ParserErrorKind::UnexpectedEnd(Group::One(idem))
    }
    /// Конструирует новый `ParserErrorKind::UnexpectedEnd` без сообщения
    #[inline]
    pub fn unexpected_end() -> Self {
        ParserErrorKind::UnexpectedEnd(Group::None)
    }
    /// Конструирует новый `ParserErrorKind::ExpectedGot`, содержащий инофрмацию о типе ожидаемого и полученного токенов
    #[inline]
    pub fn expected_got(expected: ParserErrorTokenInfo<'source>, got: ParserErrorTokenInfo<'source>) -> Self {
        ParserErrorKind::ExpectedGot(Group::One(expected), got)
    }
    /// Конструирует новый `ParserErrorKind::NomError`, содержащий сообщение об ошибке комбинатора парсеров
    #[inline]
    pub fn custom_error<A: ToString>(msg: A) -> Self {
        ParserErrorKind::CustomError(Group::One(msg.to_string()))
    }
}

impl<'source> Appendable for ParserErrorKind<'source> {
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
impl<'source> Display for ParserErrorKind<'source> {
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
            &ParserErrorKind::CustomError(ref messages) => display_list(f, &messages.extract_into_vec()),
        }
    }
}

impl<'source> Default for ParserErrorKind<'source> {
    /// Нейтральным значением `ParserErrorKind` является `ParserErrorKind::CustomError(Group::None)`
    fn default() -> Self {
        ParserErrorKind::CustomError(Group::None)
    }
}

impl<'source> IntoStatic for ParserErrorKind<'source> {
    type Result = ParserErrorKind<'static>;
    fn into_static(self) -> Self::Result {
        match self {
            ParserErrorKind::UnexpectedEnd(group) => ParserErrorKind::UnexpectedEnd(group.into_static()),
            ParserErrorKind::ExpectedGot(group, info) => ParserErrorKind::ExpectedGot(
                group.into_static(),
                info.into_static(),
            ),
            ParserErrorKind::CustomError(group) => ParserErrorKind::CustomError(group),
        }
    }
}

/// Одиночная ошибка разбора. Применяется как элемент `ParserError`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParserErrorItem<'source> {
    pub kind: ParserErrorKind<'source>,
    pub pos: Option<SymbolPosition>,
}

impl<'source> ParserErrorItem<'source> {
    /// Конструирует новую единицу ошибки из типа и позиции
    #[inline]
    fn new(kind: ParserErrorKind<'source>, pos: SymbolPosition) -> Self {
        Self {
            kind,
            pos: Some(pos),
        }
    }
    /// Конструирует новую единицу ошибки из типа, но без позиции
    #[inline]
    fn new_without_pos(kind: ParserErrorKind<'source>) -> Self {
        Self {
            kind,
            pos: None,
        }
    }
}

impl<'source> Appendable for ParserErrorItem<'source> {
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
impl<'source> Display for ParserErrorItem<'source> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "{}", self.kind)?;
        if let &Some(ref pos) = &self.pos {
            write!(f, " on {}", pos)?;
        }
        Ok(())
    }
}

impl<'source> PartialOrd for ParserErrorItem<'source> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.pos.partial_cmp(&other.pos)
    }
}

impl<'source> Ord for ParserErrorItem<'source> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("Trying to sort error from different modules")
    }
}

/// Ошибка разбора. Может содержать несколько `ParserErrorItem`.
pub type ParserError<'source> = Group<ParserErrorItem<'source>>;

/// Конструирует единичную ошибку из типа и позиции
pub fn new_error<'source>(kind: ParserErrorKind<'source>, pos: SymbolPosition) -> ParserError<'source> {
    Group::One(
        ParserErrorItem::new(kind, pos)
    )
}

/// Конструирует единичную ошибку из типа, но без позиции
pub fn new_error_without_pos<'source>(kind: ParserErrorKind<'source>) -> ParserError<'source> {
    Group::One(
        ParserErrorItem::new_without_pos(kind)
    )
}

/// Типаж Display у `ParserError` служит для отображения группы ошибок в человекочитаемом виде
impl<'source> Display for ParserError<'source> {
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
            }
        }
    }
}

impl<'source> IntoStatic for ParserErrorItem<'source> {
    type Result = ParserErrorItem<'static>;
    fn into_static(self) -> Self::Result {
        let ParserErrorItem { pos, kind } = self;
        ParserErrorItem { pos, kind: kind.into_static() }
    }
}
