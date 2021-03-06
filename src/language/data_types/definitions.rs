#![allow(unused_imports)]

use helpers::{
    as_unique_identifier,
    Assertion,
    BlockFormatter,
    Format,
    Generate,
    parse_index,
    Path,
    PathBuf,
    Resolve,
    SimpleFormatter,
    SyncRef,
    TSQLParameters,
};
use indexmap::IndexMap;
use language::ItemPath;
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use project_analysis::{
    Item,
    Module,
    SemanticError,
    SemanticErrorKind,
    SemanticItemType,
};
use std::{
    fmt::{
        self,
        Write,
    },
    mem::replace,
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberType {
    Bit {
        size: Option<u32>,
    },
    Boolean,
    Integer {
        size: u8,
        unsigned: bool,
        zerofill: bool,
    },
    Decimal {
        size: Option<(u32, Option<u32>)>,
        unsigned: bool,
        zerofill: bool,
    },
    Float {
        size: Option<(u32, u32)>,
        double: bool,
    },
}

#[inline]
pub fn int_class(size: u32) -> &'static str {
    match size {
        0..=1 => "bit",
        2..=8 => "tinyint",
        9..=16 => "smallint",
        17..=32 => "int",
        33..=64 => "bigint",
        _ => panic!("{} is too big size for integer in ms-sql", size),
    }
}

#[inline]
pub fn int_class_ts_mssql(size: u32) -> &'static str {
    match size {
        0..=1 => "Bit",
        2..=8 => "TinyInt",
        9..=16 => "SmallInt",
        17..=32 => "Int",
        33..=64 => "BigInt",
        _ => panic!("{} is too big size for integer in ms-sql", size),
    }
}

impl NumberType {
    pub fn can_cast(&self, target: &NumberType) -> bool {
        match self {
            NumberType::Bit { size } => {
                let self_size = size.unwrap_or(1);
                if let NumberType::Bit { size } = target {
                    let other_size = size.unwrap_or(1);
                    return self_size <= other_size;
                }
            }
            NumberType::Boolean => {
                if let NumberType::Boolean = target { return true; }
            }
            NumberType::Integer { size: self_size, unsigned: self_unsigned, zerofill: _ } => {
                if let NumberType::Integer { size, unsigned, zerofill: _ } = target {
                    if !*self_unsigned && *unsigned { return false; }
                    return *self_size <= *size;
                }
            }
            NumberType::Decimal { size, unsigned: self_unsigned, zerofill: _ } => {
                let self_size = match *size {
                    Some((m, d)) => match d {
                        Some(d) => (m, d),
                        None => (m, 30),
                    },
                    None => (65, 30),
                };
                if let NumberType::Decimal { size, unsigned, zerofill: _ } = target {
                    let other_size = match *size {
                        Some((m, d)) => match d {
                            Some(d) => (m, d),
                            None => (m, 30),
                        },
                        None => (65, 30),
                    };
                    if !*self_unsigned && *unsigned { return false; }
                    return (self_size.0 <= other_size.0) && (self_size.1 <= other_size.1);
                }
            }
            NumberType::Float { size: _, double: self_double } => {
                if let NumberType::Float { size: _, double } = target {
                    return !*self_double || *double;
                }
            }
        }
        false
    }
    pub fn check(&self) -> Result<(), SemanticErrorKind> {
        match self {
            NumberType::Bit { size } => {
                if size.unwrap_or(1) > 64 {
                    return Err(SemanticErrorKind::NotSupportedYet {
                        feature: "long bit sets",
                    });
                }
            }
            NumberType::Integer { size, .. } => {
                if *size > 64 {
                    return Err(SemanticErrorKind::NotSupportedYet {
                        feature: "big numbers",
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl fmt::Display for NumberType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NumberType::Bit { size } => {
                write!(f, "bit")?;
                if let Some(size) = size {
                    write!(f, "({})", size)?;
                }
                Ok(())
            }
            NumberType::Boolean => write!(f, "boolean"),
            NumberType::Integer { size, unsigned, zerofill } => {
                if *unsigned { write!(f, "unsigned ")?; }
                if *zerofill { write!(f, "zerofill ")?; }
                write!(f, "integer({})", size)
            }
            NumberType::Decimal { size, unsigned, zerofill } => {
                if *unsigned { write!(f, "unsigned ")?; }
                if *zerofill { write!(f, "zerofill ")?; }
                write!(f, "decimal")?;
                if let Some((size_a, size_b)) = size {
                    write!(f, "({}", size_a)?;
                    if let Some(size_b) = size_b {
                        write!(f, ", {}", size_b)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            NumberType::Float { size, double } => {
                if *double {
                    write!(f, "double")?;
                } else {
                    write!(f, "float")?;
                }
                if let Some((size_a, size_b)) = size {
                    write!(f, "({}, {})", size_a, size_b)?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> Format<TSQLParameters<'a>> for NumberType {
    fn fmt(&self, f: &mut impl fmt::Write, _parameters: TSQLParameters<'a>) -> fmt::Result {
        match self {
            NumberType::Bit { size } => {
                f.write_str(int_class(size.unwrap_or(1)))
            }
            NumberType::Boolean => f.write_str("bit"),
            NumberType::Integer { size, .. } => f.write_str(int_class((*size).into())),
            NumberType::Decimal { size, .. } => match size {
                None => f.write_str("decimal"),
                Some((p, None)) => write!(f, "decimal({})", p),
                Some((p, Some(s))) => write!(f, "decimal({}, {})", p, s),
            }
            NumberType::Float { double, .. } => {
                let class = if *double { "double" } else { "float" };
                f.write_str(class)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DateTimeType {
    Date,
    Time {
        precision: Option<u32>,
    },
    Datetime {
        precision: Option<u32>,
    },
    Timestamp {
        precision: Option<u32>,
    },
}

impl fmt::Display for DateTimeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DateTimeType::Date => write!(f, "date"),
            DateTimeType::Time { precision } => {
                write!(f, "time")?;
                if let Some(precision) = precision {
                    write!(f, "({})", precision)?;
                }
                Ok(())
            }
            DateTimeType::Datetime { precision } => {
                write!(f, "datetime")?;
                if let Some(precision) = precision {
                    write!(f, "({})", precision)?;
                }
                Ok(())
            }
            DateTimeType::Timestamp { precision } => {
                write!(f, "timestamp")?;
                if let Some(precision) = precision {
                    write!(f, "({})", precision)?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> Format<TSQLParameters<'a>> for DateTimeType {
    fn fmt(&self, f: &mut impl fmt::Write, _parameters: TSQLParameters<'a>) -> fmt::Result {
        let (class, precision) = match self {
            DateTimeType::Date => ("date", &None),
            DateTimeType::Time { precision } => ("time", precision),
            DateTimeType::Datetime { precision } => ("datetime", precision),
            DateTimeType::Timestamp { precision } => ("timestamp", precision),
        };
        f.write_str(class)?;
        if let Some(p) = precision {
            write!(f, "({})", p)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum YearType {
    Year2,
    Year4,
}

impl YearType {
    #[inline]
    pub fn can_cast(&self, target: &YearType) -> bool {
        match self {
            &YearType::Year2 => true,
            &YearType::Year4 => *target == YearType::Year4,
        }
    }
}

impl fmt::Display for YearType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &YearType::Year2 => write!(f, "year(2)"),
            &YearType::Year4 => write!(f, "year(4)"),
        }
    }
}

impl<'a> Format<TSQLParameters<'a>> for YearType {
    fn fmt(&self, f: &mut impl fmt::Write, _parameters: TSQLParameters<'a>) -> fmt::Result {
        f.write_str("smallint")
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CharacterSet {
    Binary,
    UTF8,
}

impl fmt::Display for CharacterSet {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &CharacterSet::Binary => write!(f, "binary"),
            &CharacterSet::UTF8 => write!(f, "UTF-8"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StringType {
    Varchar {
        size: Option<u32>,
        character_set: Option<CharacterSet>,
    },
    Text {
        character_set: Option<CharacterSet>,
    },
}

impl StringType {
    #[inline]
    pub fn can_cast(&self, target: &StringType) -> bool {
        match self {
            StringType::Varchar { size, character_set: _ } => {
                let self_size = size.unwrap_or(255);
                match target {
                    StringType::Varchar { size, character_set: _ } => {
                        let size = size.unwrap_or(255);
                        self_size <= size
                    }
                    StringType::Text { character_set: _ } => true,
                }
            }
            StringType::Text { character_set: _ } => {
                match target {
                    StringType::Varchar { size: _, character_set: _ } => false,
                    StringType::Text { character_set: _ } => true,
                }
            }
        }
    }
}

impl fmt::Display for StringType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StringType::Varchar { size, character_set } => {
                write!(f, "varchar")?;
                if let Some(size) = size {
                    write!(f, "({})", size)?;
                }
                if let Some(character_set) = character_set {
                    write!(f, " character set {}", character_set)?;
                }
                Ok(())
            }
            StringType::Text { character_set } => {
                write!(f, "text")?;
                if let Some(character_set) = character_set {
                    write!(f, " character set {}", character_set)?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> Format<TSQLParameters<'a>> for StringType {
    fn fmt(&self, f: &mut impl fmt::Write, _parameters: TSQLParameters<'a>) -> fmt::Result {
        match self {
            StringType::Varchar { size, .. } => {
                f.write_str("nvarchar")?;
                if let Some(size) = size {
                    write!(f, "({})", size)
                } else {
                    Ok(())
                }
            }
            StringType::Text { .. } => f.write_str("ntext"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PrimitiveDataType {
    Null,
    Number(NumberType),
    DateTime(DateTimeType),
    Year(YearType),
    String(StringType),
}

impl PrimitiveDataType {
    pub fn can_cast(&self, target: &PrimitiveDataType) -> bool {
        match self {
            PrimitiveDataType::Null => return *target == PrimitiveDataType::Null,
            PrimitiveDataType::Number(self_number) => {
                if let PrimitiveDataType::Number(number) = target {
                    return self_number.can_cast(number);
                }
            }
            PrimitiveDataType::DateTime(_) => unimplemented!(),
            PrimitiveDataType::Year(self_year) => {
                if let PrimitiveDataType::Year(year) = target {
                    return self_year.can_cast(year);
                }
            }
            PrimitiveDataType::String(self_string) => {
                if let PrimitiveDataType::String(string) = target {
                    return self_string.can_cast(string);
                }
            }
        }
        false
    }
    #[inline]
    pub fn check(&self) -> Result<(), SemanticErrorKind> {
        match self {
            PrimitiveDataType::Number(x) => x.check(),
            _ => Ok(()),
        }
    }
    pub fn fmt_ts_mssql(&self, f: &mut impl Write) -> fmt::Result {
        match self {
            PrimitiveDataType::Null => f.write_str("Bit"),
            PrimitiveDataType::Number(NumberType::Bit { size }) => {
                f.write_str(int_class_ts_mssql(size.unwrap_or(1)))
            }
            PrimitiveDataType::Number(NumberType::Boolean) => f.write_str("Bit"),
            PrimitiveDataType::Number(NumberType::Integer { size, .. }) => {
                f.write_str(int_class_ts_mssql((*size).into()))
            }
            PrimitiveDataType::Number(NumberType::Decimal { size, .. }) => {
                match size {
                    Some((precision, Some(scale))) => write!(f, "Decimal({}, {})", precision, scale),
                    Some((precision, None)) => write!(f, "Decimal({})", precision),
                    None => write!(f, "Decimal"),
                }
            }
            PrimitiveDataType::Number(NumberType::Float { size, double }) => {
                match size {
                    Some((precision, scale)) => write!(f, "Float({}, {})", precision, scale),
                    None => if *double {
                        write!(f, "Float(24)")
                    } else {
                        write!(f, "Float(53)")
                    }
                }
            }
            PrimitiveDataType::DateTime(date_time_type) => {
                let (class, precision) = match date_time_type {
                    DateTimeType::Date => ("Date", &None),
                    DateTimeType::Time { precision } => ("Time", precision),
                    DateTimeType::Datetime { precision } => ("DateTime", precision),
                    DateTimeType::Timestamp { precision } => ("DateTime2", precision),
                };
                if let Some(precision) = precision {
                    write!(f, "{}({})", class, precision)
                } else {
                    write!(f, "{}", class)
                }
            },
            PrimitiveDataType::Year(_) => f.write_str("SmallInt"),
            PrimitiveDataType::String(StringType::Varchar { size, .. }) => {
                if let Some(size) = size {
                    write!(f, "NVarChar({})", size)
                } else {
                    f.write_str("NVarChar")
                }
            }
            PrimitiveDataType::String(StringType::Text { .. }) => f.write_str("NText"),
        }
    }
}

// TODO Удалить impl fmt::Display у всех сущностей, которым это не нужно. В частности, у типов данных.
impl fmt::Display for PrimitiveDataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrimitiveDataType::Null => write!(f, "null"),
            PrimitiveDataType::Number(primitive) => write!(f, "{}", primitive),
            PrimitiveDataType::DateTime(primitive) => write!(f, "{}", primitive),
            PrimitiveDataType::Year(primitive) => write!(f, "{}", primitive),
            PrimitiveDataType::String(primitive) => write!(f, "{}", primitive),
        }
    }
}

impl<'a> Format<TSQLParameters<'a>> for PrimitiveDataType {
    fn fmt(&self, f: &mut impl fmt::Write, parameters: TSQLParameters<'a>) -> fmt::Result {
        match self {
            PrimitiveDataType::Null => f.write_str("null"),
            PrimitiveDataType::Number(x) => Format::<TSQLParameters>::fmt(x, f, parameters),
            PrimitiveDataType::DateTime(x) => Format::<TSQLParameters>::fmt(x, f, parameters),
            PrimitiveDataType::Year(x) => Format::<TSQLParameters>::fmt(x, f, parameters),
            PrimitiveDataType::String(x) => Format::<TSQLParameters>::fmt(x, f, parameters),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttributeAST<'source> {
    pub name: Identifier<'source>,
    pub arguments: Option<Vec<Identifier<'source>>>,
}

impl<'a, 'source> Into<Attribute> for &'a AttributeAST<'source> {
    fn into(self) -> Attribute {
        Attribute {
            name: self.name.text().to_string(),
            arguments: match &self.arguments {
                Some(args) => Some(args.iter()
                    .map(|s| s.text().to_string())
                    .collect()
                ),
                None => None,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attribute {
    pub name: String,
    pub arguments: Option<Vec<String>>,
}

#[inline]
pub fn find_attribute_ast<'a, 'source>(attributes: &'a [AttributeAST<'source>], name: &str) -> Option<&'a AttributeAST<'source>> {
    for attribute in attributes.iter() {
        if attribute.name == name {
            return Some(attribute);
        }
    }
    None
}

#[inline]
pub fn find_attribute<'a, 'source>(attributes: &'a [Attribute], name: &str) -> Option<&'a Attribute> {
    for attribute in attributes.iter() {
        if attribute.name == name {
            return Some(attribute);
        }
    }
    None
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FieldAST<'source> {
    pub attributes: Vec<AttributeAST<'source>>,
    pub field_type: DataTypeAST<'source>,
    pub position: ItemPosition,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    pub attributes: Vec<Attribute>,
    pub field_type: DataType,
}

impl<'source> Assertion for FieldAST<'source> {
    fn assert(&self, other: &FieldAST) {
        assert_eq!(self.attributes, other.attributes);
        self.field_type.assert(&other.field_type);
    }
}

impl<'source> Resolve<SyncRef<Module>> for FieldAST<'source> {
    type Result = Field;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        let field_type = self.field_type.resolve(ctx)?;
        let attributes = self.attributes.iter()
            .map(|attr| attr.into())
            .collect();
        Ok(Field {
            attributes,
            field_type,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompoundDataTypeAST<'source> {
    Structure(Vec<(Identifier<'source>, FieldAST<'source>)>),
    Tuple(Vec<FieldAST<'source>>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CompoundDataType {
    Structure(Arc<IndexMap<String, Field>>),
    Tuple(Arc<Vec<Field>>),
}

impl CompoundDataType {
    pub fn can_cast(&self, target: &CompoundDataType) -> bool {
        match self {
            CompoundDataType::Structure(self_fields) => {
                if let CompoundDataType::Structure(fields) = target {
                    for (name, field) in fields.iter() {
                        let self_field = match self_fields.get(name.as_str()) {
                            Some(field) => field,
                            None => return false,
                        };
                        if !self_field.field_type.can_cast(&field.field_type) {
                            return false;
                        }
                    }
                    return true;
                }
            }
            CompoundDataType::Tuple(self_fields) => {
                if let CompoundDataType::Tuple(fields) = target {
                    for (i, field) in fields.iter().enumerate() {
                        let self_field = match self_fields.get(i) {
                            Some(field) => field,
                            None => return false,
                        };
                        if !self_field.field_type.can_cast(&field.field_type) {
                            return false;
                        }
                    }
                    return true;
                }
            }
        }
        false
    }
    #[inline]
    pub fn get_field(&self, index: usize) -> Option<&Field> {
        match self {
            CompoundDataType::Structure(fields) => fields.get_index(index)
                .map(|(_, field)| field),
            CompoundDataType::Tuple(fields) => fields.get(index),
        }
    }
    #[inline]
    pub fn field_len(&self) -> usize {
        match self {
            CompoundDataType::Structure(fields) => fields.len(),
            CompoundDataType::Tuple(fields) => fields.len(),
        }
    }
}

impl<'source> Assertion for CompoundDataTypeAST<'source> {
    fn assert(&self, other: &CompoundDataTypeAST) {
        match self {
            CompoundDataTypeAST::Structure(fields) => {
                let mut other_fields_iter = match_it!(other,
                    CompoundDataTypeAST::Structure(fields) => { fields.iter() }
                );
                for (field_name, field) in fields.iter() {
                    let (other_field_name, other_field) = other_fields_iter.next()
                        .expect("Field lists should have equal sizes");
                    assert_eq!(field_name, other_field_name);
                    field.assert(other_field);
                }
                assert_eq!(other_fields_iter.next(), None);
            }
            CompoundDataTypeAST::Tuple(fields) => {
                let mut other_fields_iter = match_it!(other,
                    CompoundDataTypeAST::Tuple(fields) => { fields.iter() }
                );
                for field in fields.iter() {
                    let other_field = other_fields_iter.next()
                        .expect("Field lists should have equal sizes");
                    field.assert(other_field);
                }
                assert_eq!(other_fields_iter.next(), None);
            }
        }
    }
}

impl<'source> Resolve<SyncRef<Module>> for CompoundDataTypeAST<'source> {
    type Result = CompoundDataType;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        match self {
            CompoundDataTypeAST::Structure(fields) => Ok(CompoundDataType::Structure(
                match as_unique_identifier(fields.clone()) {
                    Ok(map) => Arc::new(map.resolve(ctx)?),
                    Err(name) => return SemanticError::duplicate_definition(
                        name.item_pos(),
                        name.text().to_string(),
                        SemanticItemType::Field,
                    )
                        .into_err_vec()
                }
            )),
            CompoundDataTypeAST::Tuple(fields) => Ok(CompoundDataType::Tuple(Arc::new(fields.resolve(ctx)?))),
        }
    }
}

impl<'source> Resolve<SyncRef<Module>> for Vec<(Identifier<'source>, FieldAST<'source>)> {
    type Result = Vec<(String, Field)>;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        self.iter()
            .map(|(name, field)| {
                let field = field.resolve(ctx)?;
                Ok((name.text().to_string(), field))
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DataTypeAST<'source> {
    pub pos: ItemPosition,
    pub body: DataTypeASTBody<'source>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataTypeASTBody<'source> {
    Array(Box<DataTypeAST<'source>>),
    Compound(CompoundDataTypeAST<'source>),
    Primitive(PrimitiveDataType),
    Reference(ItemPath),
}

impl<'source> DataTypeAST<'source> {
    pub fn array(self) -> Self {
        let pos = self.pos;
        Self {
            pos,
            body: DataTypeASTBody::Array(box self),
        }
    }
}

impl<'source> Assertion for DataTypeAST<'source> {
    fn assert(&self, other_data_type: &DataTypeAST) {
        let other_body = &other_data_type.body;
        match &self.body {
            DataTypeASTBody::Compound(compound_type) => {
                match_it!(other_body, DataTypeASTBody::Compound(other_compound_type) => {
                    compound_type.assert(other_compound_type);
                });
            }
            DataTypeASTBody::Reference(path) => {
                match_it!(other_body, DataTypeASTBody::Reference(other_path) => {
                    assert_eq!(path.path, other_path.path);
                });
            }
            other => assert_eq!(other, other_body),
        }
    }
}

impl<'source> Assertion<str> for DataTypeAST<'source> {
    fn assert(&self, other: &str) {
        let tokens = ::lexeme_scanner::Scanner::scan(other)
            .expect("Scanner result must be ok");
        let other_data_type = ::parser_basics::parse(
            tokens.as_slice(),
            ::language::data_type,
        ).expect("Parser result must be ok");
        self.assert(&other_data_type);
    }
}

impl<'a, 'source> Assertion<&'a str> for DataTypeAST<'source> {
    fn assert(&self, other: &&'a str) {
        self.assert(*other)
    }
}

impl<'source> Resolve<SyncRef<Module>> for DataTypeAST<'source> {
    type Result = DataType;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        match &self.body {
            DataTypeASTBody::Array(sub_type) => Ok(DataType::Array(
                Arc::new((**sub_type).resolve(ctx)?)
            )),
            DataTypeASTBody::Compound(value) => Ok(DataType::Compound(value.resolve(ctx)?)),
            DataTypeASTBody::Primitive(value) => {
                if let Err(kind) = value.check() {
                    return Err(vec![SemanticError::new(self.pos, kind)]);
                }
                Ok(DataType::Primitive(value.clone()))
            }
            DataTypeASTBody::Reference(path) => {
                let item = match ctx.get_item(path.path.as_path(), &mut vec![]) {
                    Some(item) => item,
                    None => return SemanticError::unresolved_item(path.pos, path.path.clone()).into_err_vec(),
                };
                let item_type = item.get_type();
                if item_type != SemanticItemType::DataType {
                    return SemanticError::expected_item_of_another_type(
                        path.pos,
                        SemanticItemType::DataType,
                        item_type,
                    )
                        .into_err_vec();
                }
                Ok(DataType::Reference(item))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum DataType {
    Array(Arc<DataType>),
    Compound(CompoundDataType),
    Primitive(PrimitiveDataType),
    Reference(SyncRef<Item>),
    Void,
}

pub const BOOLEAN_TYPE: DataType = DataType::Primitive(PrimitiveDataType::Number(NumberType::Boolean));

impl DataType {
    pub fn property_type(&self, pos: ItemPosition, prop: Path) -> Result<DataType, SemanticError> {
        let mut path = prop;
        let field_name = match path.pop_left() {
            Some(name) => name,
            None => return Ok(self.clone()),
        };
        match self {
            DataType::Compound(CompoundDataType::Structure(fields)) => {
                if let Some(field) = fields.get(field_name) {
                    return field.field_type.property_type(pos, path);
                }
            }
            DataType::Compound(CompoundDataType::Tuple(fields)) => {
                if let Some(component) = parse_index(field_name) {
                    if let Some(field) = fields.get(component) {
                        return field.field_type.property_type(pos, path);
                    }
                }
            }
            DataType::Reference(item) => {
                let item = item.read();
                if let Some(data_type) = item.get_data_type() {
                    return data_type.body.property_type(pos, prop);
                }
            }
            _ => {}
        }
        Err(SemanticError::wrong_property(pos, field_name.to_string()))
    }
    pub fn can_cast(&self, target: &DataType) -> bool {
        if let DataType::Reference(reference) = target {
            let guard = reference.read();
            let data_type = match guard.get_data_type() {
                Some(data_type) => data_type,
                None => return false,
            };
            return self.can_cast(&data_type.body);
        }
        match self {
            DataType::Array(self_subtype) => {
                if let DataType::Array(subtype) = target {
                    return self_subtype.can_cast(&*subtype);
                }
            }
            DataType::Compound(self_subtype) => {
                if let DataType::Compound(subtype) = target {
                    return self_subtype.can_cast(&*subtype);
                }
            }
            DataType::Primitive(self_subtype) => {
                if let DataType::Primitive(subtype) = target {
                    return self_subtype.can_cast(&*subtype);
                }
            }
            DataType::Reference(reference) => {
                let guard = reference.read();
                let data_type = match guard.get_data_type() {
                    Some(data_type) => data_type,
                    None => return false,
                };
                return data_type.body.can_cast(target);
            }
            DataType::Void => return *target == DataType::Void,
        }
        false
    }
    pub fn should_cast_to(&self, pos: ItemPosition, target: &DataType) -> Result<(), SemanticError> {
        if self.can_cast(target) {
            Ok(())
        } else {
            Err(SemanticError::cannot_cast_type(
                pos,
                self.clone(),
                target.clone(),
            ))
        }
    }
    pub fn get_field_type(&self, index: usize) -> Option<DataType> {
        let one = match self {
            DataType::Array(item) => &*item,
            DataType::Compound(compound) => return compound.get_field(index)
                .map(|field| field.field_type.clone()),
            DataType::Primitive(_) => self,
            DataType::Reference(item) => {
                let item = item.read();
                let def = item.get_data_type()?;
                return def.body.get_field_type(index);
            }
            DataType::Void => self,
        };
        if index == 0 { Some(one.clone()) } else { None }
    }
    pub fn field_len(&self) -> usize {
        match self {
            DataType::Array(_) |
            DataType::Primitive(_) |
            DataType::Void => 1,
            DataType::Compound(compound) => compound.field_len(),
            DataType::Reference(item) => {
                let item = item.read();
                let def = match item.get_data_type() {
                    Some(item) => item,
                    None => return 0,
                };
                return def.body.field_len();
            }
        }
    }
    pub fn make_primitives(&self, prefix: PathBuf, target: &mut Vec<FieldPrimitive>) {
        match self {
            DataType::Array(sub_type) => {
                let mut sub_prefix = prefix;
                sub_prefix.push("[]");
                sub_type.make_primitives(sub_prefix, target);
            }
            DataType::Primitive(primitive) => {
                target.push(FieldPrimitive {
                    path: prefix,
                    field_type: primitive.clone(),
                });
            }
            DataType::Void => {
                target.push(FieldPrimitive {
                    path: prefix,
                    field_type: PrimitiveDataType::Number(NumberType::Bit {
                        size: Some(0),
                    }),
                });
            }
            DataType::Compound(CompoundDataType::Tuple(fields)) => {
                for (i, field) in fields.iter().enumerate() {
                    let mut path = prefix.clone();
                    if path.push_fmt(format_args!("component{}", i)).is_ok() {
                        field.field_type.make_primitives(path, target);
                    }
                }
            }
            DataType::Compound(CompoundDataType::Structure(fields)) => {
                for (field_name, field) in fields.iter() {
                    let mut path = prefix.clone();
                    path.push(field_name.as_str());
                    field.field_type.make_primitives(path, target);
                }
            }
            DataType::Reference(item) => {
                let item = item.read();
                if let Some(data_type) = item.get_data_type() {
                    data_type.body.make_primitives(prefix, target);
                };
            }
        }
    }
    pub fn primitives(&self, prefix: PathBuf) -> Vec<FieldPrimitive> {
        let mut result = Vec::new();
        self.make_primitives(prefix, &mut result);
        result
    }
    pub fn can_be_table(&self) -> bool {
        match self {
            DataType::Array(_) |
            DataType::Compound(_) => true,
            DataType::Reference(item) => {
                let item = item.read();
                if let Some(data_type) = item.get_data_type() {
                    data_type.body.can_be_table()
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    pub fn make_table_type(&self, prefix: PathBuf, consumer: &mut Vec<FieldPrimitive>) -> bool {
        match self {
            DataType::Array(sub_type) => {
                sub_type.make_primitives(prefix, consumer);
                true
            }
            DataType::Compound(_) => {
                self.make_primitives(prefix, consumer);
                true
            }
            DataType::Reference(item) => {
                let item = item.read();
                if let Some(data_type) = item.get_data_type() {
                    data_type.body.make_table_type(prefix, consumer)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    #[inline]
    pub fn as_table_type(&self, prefix: PathBuf) -> Option<Vec<FieldPrimitive>> {
        let mut result = Vec::new();
        if self.make_table_type(prefix, &mut result) {
            Some(result)
        } else {
            None
        }
    }
    #[inline]
    pub fn as_primitive(&self) -> Option<PrimitiveDataType> {
        match self {
            DataType::Primitive(x) => Some(x.clone()),
            DataType::Reference(item) => {
                let item_guard = item.read();
                let references_data_type = item_guard.get_data_type()?;
                references_data_type.body.as_primitive()
            }
            DataType::Void => {
                Some(PrimitiveDataType::Number(NumberType::Bit {
                    size: Some(0),
                }))
            }
            _ => None,
        }
    }
    #[inline]
    pub fn as_array(&self) -> Option<&Arc<DataType>> {
        match self {
            DataType::Array(sub_type) => Some(sub_type),
            _ => None,
        }
    }
    pub fn fmt(
        &self,
        f: &mut SimpleFormatter,
    ) -> fmt::Result {
        match self {
            DataType::Array(sub_type) => {
                sub_type.fmt(f)?;
                f.write_str("[]")
            }
            DataType::Compound(CompoundDataType::Structure(fields)) => {
                if fields.is_empty() {
                    return f.write_str("{}");
                }
                writeln!(f, "{{")?;
                {
                    let mut sub_f = f.sub_block();
                    for (field_name, field) in fields.iter() {
                        write!(sub_f, "{}: ", field_name)?;
                        field.field_type.fmt(&mut sub_f)?;
                        writeln!(sub_f, ",")?;
                    }
                }
                write!(f, "}}")
            }
            DataType::Compound(CompoundDataType::Tuple(fields)) => {
                if fields.is_empty() {
                    return f.write_str("[]");
                }
                writeln!(f, "[")?;
                {
                    let mut sub_f = f.sub_block();
                    for field in fields.iter() {
                        field.field_type.fmt(&mut sub_f)?;
                        writeln!(sub_f, ",")?;
                    }
                }
                write!(f, "]")
            }
            DataType::Primitive(PrimitiveDataType::Null) => {
                f.write_str("null")
            }
            DataType::Primitive(PrimitiveDataType::Number(NumberType::Bit { size })) => {
                f.write_str(match size {
                    Some(0) => "void",
                    Some(1) => "boolean",
                    _ => "number",
                })
            }
            DataType::Primitive(PrimitiveDataType::Number(NumberType::Boolean)) => {
                f.write_str("boolean")
            }
            DataType::Primitive(PrimitiveDataType::Number(_)) => {
                f.write_str("number")
            }
            DataType::Primitive(PrimitiveDataType::DateTime(_)) => {
                f.write_str("Date")
            }
            DataType::Primitive(PrimitiveDataType::Year(_)) => {
                f.write_str("number")
            }
            DataType::Primitive(PrimitiveDataType::String(_)) => {
                f.write_str("string")
            }
            DataType::Reference(reference) => {
                let guard = reference.read();
                f.write_str(
                    guard.get_path()
                        .as_path()
                        .into_new_buf(".")
                        .data.as_str()
                )
            }
            DataType::Void => {
                f.write_str("void")
            }
        }
    }
    pub fn fmt_export(
        &self,
        f: &mut SimpleFormatter,
        name: &str,
    ) -> fmt::Result {
        match self {
            DataType::Array(_) |
            DataType::Primitive(_) |
            DataType::Reference(_) |
            DataType::Void => {
                write!(f, "export type {} = ", name)?;
                self.fmt(f)?;
                writeln!(f, ";")
            }
            DataType::Compound(_) => {
                write!(f, "export interface {} ", name)?;
                self.fmt(f)?;
                writeln!(f, "")
            }
        }
    }
    pub fn fmt_result_bind(
        &self,
        f: &mut SimpleFormatter,
        variable: &str,
        prefix: Path,
    ) -> fmt::Result {
        match self {
            DataType::Array(_) => panic!("Array-type cannot be bind"),
            DataType::Compound(CompoundDataType::Tuple(fields)) => {
                writeln!(f, "[")?;
                {
                    let mut sub_f = f.sub_block();
                    for (i, field) in fields.iter().enumerate() {
                        let mut field_prefix = prefix.into_buf();
                        field_prefix.push_fmt(format_args!("component{}", i))?;
                        field.field_type.fmt_result_bind(
                            &mut sub_f,
                            variable,
                            field_prefix.as_path(),
                        )?;
                        writeln!(sub_f, ",")?;
                    }
                }
                write!(f, "]")
            }
            DataType::Compound(CompoundDataType::Structure(fields)) => {
                writeln!(f, "{{")?;
                {
                    let mut sub_f = f.sub_block();
                    for (name, field) in fields.iter() {
                        let mut field_prefix = prefix.into_buf();
                        field_prefix.push(&*name);
                        write!(sub_f, "{}: ", name)?;
                        field.field_type.fmt_result_bind(
                            &mut sub_f,
                            variable,
                            field_prefix.as_path(),
                        )?;
                        writeln!(sub_f, ",")?;
                    }
                }
                write!(f, "}}")
            }
            DataType::Primitive(_) => {
                write!(
                    f,
                    "{var}['{path}']",
                    var = variable,
                    path = prefix,
                )
            }
            DataType::Reference(item) => {
                item.read()
                    .get_data_type()
                    .expect("Wrong references are not allowed at generate-time")
                    .body.fmt_result_bind(
                        f,
                        variable,
                        prefix
                    )
            }
            DataType::Void => f.write_str("void 0"),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Array(subtype) => write!(f, "[{}]", subtype),
            DataType::Compound(CompoundDataType::Structure(fields)) => {
                write!(f, "{{")?;
                let mut fields = fields.iter();
                if let Some((name, field)) = fields.next() {
                    write!(f, "{}: {}", name, field.field_type)?;
                }
                for (name, field) in fields {
                    write!(f, ", {}: {}", name, field.field_type)?;
                }
                write!(f, "}}")
            }
            DataType::Compound(CompoundDataType::Tuple(components)) => {
                write!(f, "(")?;
                let mut components = components.iter();
                if let Some(component) = components.next() {
                    write!(f, "{}", component.field_type)?;
                }
                for component in components {
                    write!(f, ", {}", component.field_type)?;
                }
                write!(f, ")")
            }
            DataType::Primitive(primitive) => write!(f, "{}", primitive),
            DataType::Reference(refer) => {
                let reference = refer.read();
                match reference.get_data_type() {
                    Some(def) => write!(f, "{}", def.body),
                    None => write!(f, "<not a type>"),
                }
            }
            DataType::Void => write!(f, "!"),
        }
    }
}

impl PartialEq for DataType {
    fn eq(&self, rhs: &DataType) -> bool {
        if let DataType::Reference(item) = rhs {
            let item_guard = item.read();
            return match item_guard.get_data_type() {
                Some(rhs) => *self == rhs.body,
                None => false,
            };
        }
        match self {
            DataType::Array(lhs_sub_type) => if let DataType::Array(rhs_sub_type) = rhs {
                *lhs_sub_type == *rhs_sub_type
            } else { false }
            DataType::Compound(lhs_compound) => if let DataType::Compound(rhs_compound) = rhs {
                *lhs_compound == *rhs_compound
            } else { false }
            DataType::Primitive(lhs_primitive) => if let DataType::Primitive(rhs_primitive) = rhs {
                *lhs_primitive == *rhs_primitive
            } else { false }
            DataType::Reference(item) => {
                let item_guard = item.read();
                match item_guard.get_data_type() {
                    Some(lhs) => lhs.body == *rhs,
                    None => false,
                }
            }
            DataType::Void => if let DataType::Void = rhs { true } else { false }
        }
    }
}

impl Eq for DataType {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldPrimitive {
    pub path: PathBuf,
    pub field_type: PrimitiveDataType,
}
