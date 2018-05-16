#![allow(unused_imports)]

use std::mem::replace;
use std::fmt;
use std::sync::Arc;
use indexmap::IndexMap;
use helpers::Assertion;
//use helpers::IntoStatic;
use helpers::{
    as_unique_identifier,
    Path,
    parse_index,
    Resolve,
    SyncRef,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use language::ItemPath;
//use project_analysis::resolve::{
//    SemanticResolve,
//ResolveContext,
//};
use project_analysis::{
    Item,
    SemanticItemType,
    SemanticError,
    Module,
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

impl NumberType {
    pub fn can_cast(&self, target: &NumberType) -> bool {
        match self {
            &NumberType::Bit { ref size } => {
                let self_size = size.unwrap_or(1);
                if let &NumberType::Bit { ref size } = target {
                    let other_size = size.unwrap_or(1);
                    return self_size <= other_size;
                }
            }
            &NumberType::Boolean => {
                if let &NumberType::Boolean = target { return true; }
            }
            &NumberType::Integer { size: ref self_size, unsigned: ref self_unsigned, zerofill: _ } => {
                if let &NumberType::Integer { ref size, ref unsigned, zerofill: _ } = target {
                    if !*self_unsigned && *unsigned { return false; }
                    return *self_size <= *size;
                }
            }
            &NumberType::Decimal { ref size, unsigned: ref self_unsigned, zerofill: _ } => {
                let self_size = match *size {
                    Some((m, d)) => match d {
                        Some(d) => (m, d),
                        None => (m, 30),
                    },
                    None => (65, 30),
                };
                if let &NumberType::Decimal { ref size, ref unsigned, zerofill: _ } = target {
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
            },
            &NumberType::Float { size: _, double: ref self_double } => {
                if let &NumberType::Float { size: _, ref double } = target {
                    return !*self_double || *double;
                }
            }
        }
        false
    }
}

impl fmt::Display for NumberType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &NumberType::Bit { ref size } => {
                write!(f, "bit")?;
                if let &Some(ref size) = size {
                    write!(f, "({})", size)?;
                }
                Ok(())
            }
            &NumberType::Boolean => write!(f, "boolean"),
            &NumberType::Integer { ref size, ref unsigned, ref zerofill } => {
                if *unsigned { write!(f, "unsigned ")?; }
                if *zerofill { write!(f, "zerofill ")?; }
                write!(f, "integer({})", size)
            }
            &NumberType::Decimal { ref size, ref unsigned, ref zerofill } => {
                if *unsigned { write!(f, "unsigned ")?; }
                if *zerofill { write!(f, "zerofill ")?; }
                write!(f, "decimal")?;
                if let &Some((ref size_a, ref size_b)) = size {
                    write!(f, "({}", size_a)?;
                    if let &Some(ref size_b) = size_b {
                        write!(f, ", {}", size_b)?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            &NumberType::Float { ref size, ref double } => {
                if *double {
                    write!(f, "double")?;
                } else {
                    write!(f, "float")?;
                }
                if let &Some((ref size_a, ref size_b)) = size {
                    write!(f, "({}, {})", size_a, size_b)?;
                }
                Ok(())
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
            &DateTimeType::Date => write!(f, "date"),
            &DateTimeType::Time { ref precision } => {
                write!(f, "time")?;
                if let &Some(ref precision) = precision {
                    write!(f, "({})", precision)?;
                }
                Ok(())
            }
            &DateTimeType::Datetime { ref precision } => {
                write!(f, "datetime")?;
                if let &Some(ref precision) = precision {
                    write!(f, "({})", precision)?;
                }
                Ok(())
            }
            &DateTimeType::Timestamp { ref precision } => {
                write!(f, "timestamp")?;
                if let &Some(ref precision) = precision {
                    write!(f, "({})", precision)?;
                }
                Ok(())
            }
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
            &StringType::Varchar { ref size, character_set: _ } => {
                let self_size = size.unwrap_or(255);
                match target {
                    &StringType::Varchar { ref size, character_set: _ } => {
                        let size = size.unwrap_or(255);
                        self_size <= size
                    }
                    &StringType::Text { character_set: _ } => true,
                }
            }
            &StringType::Text { character_set: _ } => {
                match target {
                    &StringType::Varchar { size: _, character_set: _ } => false,
                    &StringType::Text { character_set: _ } => true,
                }
            }
        }
    }
}

impl fmt::Display for StringType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &StringType::Varchar { ref size, ref character_set } => {
                write!(f, "varchar")?;
                if let &Some(ref size) = size {
                    write!(f, "({})", size)?;
                }
                if let &Some(ref character_set) = character_set {
                    write!(f, " character set {}", character_set)?;
                }
                Ok(())
            }
            &StringType::Text { ref character_set } => {
                write!(f, "text")?;
                if let &Some(ref character_set) = character_set {
                    write!(f, " character set {}", character_set)?;
                }
                Ok(())
            }
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
            &PrimitiveDataType::Null => return *target == PrimitiveDataType::Null,
            &PrimitiveDataType::Number(ref self_number) => {
                if let &PrimitiveDataType::Number(ref number) = target {
                    return self_number.can_cast(number);
                }
            }
            &PrimitiveDataType::DateTime(_) => unimplemented!(),
            &PrimitiveDataType::Year(ref self_year) => {
                if let &PrimitiveDataType::Year(ref year) = target {
                    return self_year.can_cast(year);
                }
            }
            &PrimitiveDataType::String(ref self_string) => {
                if let &PrimitiveDataType::String(ref string) = target {
                    return self_string.can_cast(string);
                }
            }
        }
        false
    }
}

impl fmt::Display for PrimitiveDataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &PrimitiveDataType::Null => write!(f, "null"),
            &PrimitiveDataType::Number(ref primitive) => write!(f, "{}", primitive),
            &PrimitiveDataType::DateTime(ref primitive) => write!(f, "{}", primitive),
            &PrimitiveDataType::Year(ref primitive) => write!(f, "{}", primitive),
            &PrimitiveDataType::String(ref primitive) => write!(f, "{}", primitive),
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
                &Some(ref args) => Some(args.iter()
                    .map(|s| s.text().to_string())
                    .collect()
                ),
                &None => None,
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
pub fn find_attribute<'a, 'source>(attributes: &'a [AttributeAST<'source>], name: &str) -> Option<&'a AttributeAST<'source>> {
    for attribute in attributes.iter() {
        if attribute.name == name {
            return Some(attribute);
        }
    }
    None
}

//impl<'source> IntoStatic for Attribute<'source> {
//    type Result = Attribute<'static>;
//    fn into_static(self) -> Self::Result {
//        let Attribute { name, arguments } = self;
//        Attribute {
//            name: name.into_static(),
//            arguments: arguments.into_static(),
//        }
//    }
//}

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
    pub position: ItemPosition,
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
            position: self.position,
        })
    }
}

//impl<'source> IntoStatic for Field<'source> {
//    type Result = Field<'static>;
//    fn into_static(self) -> Self::Result {
//        let Field {
//            attributes,
//            field_type,
//            position,
//        } = self;
//        Field {
//            attributes: attributes.into_static(),
//            field_type: field_type.into_static(),
//            position,
//        }
//    }
//}

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
            &CompoundDataType::Structure(ref self_fields) => {
                if let &CompoundDataType::Structure(ref fields) = target {
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
            &CompoundDataType::Tuple(ref self_fields) => {
                if let &CompoundDataType::Tuple(ref fields) = target {
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
}

impl<'source> Assertion for CompoundDataTypeAST<'source> {
    fn assert(&self, other: &CompoundDataTypeAST) {
        match self {
            &CompoundDataTypeAST::Structure(ref fields) => {
                let mut other_fields_iter = match_it!(other,
                    &CompoundDataTypeAST::Structure(ref fields) => { fields.iter() }
                );
                for &(ref field_name, ref field) in fields.iter() {
                    let &(ref other_field_name, ref other_field) = other_fields_iter.next()
                        .expect("Field lists should have equal sizes");
                    assert_eq!(field_name, other_field_name);
                    field.assert(other_field);
                }
                assert_eq!(other_fields_iter.next(), None);
            }
            &CompoundDataTypeAST::Tuple(ref fields) => {
                let mut other_fields_iter = match_it!(other,
                    &CompoundDataTypeAST::Tuple(ref fields) => { fields.iter() }
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

//impl<'source> IntoStatic for CompoundDataType<'source> {
//    type Result = CompoundDataType<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            CompoundDataType::Structure(fields) => CompoundDataType::Structure(fields.into_static()),
//            CompoundDataType::Tuple(fields) => CompoundDataType::Tuple(fields.into_static()),
//        }
//    }
//}

impl<'source> Resolve<SyncRef<Module>> for CompoundDataTypeAST<'source> {
    type Result = CompoundDataType;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        match self {
            &CompoundDataTypeAST::Structure(ref fields) => Ok(CompoundDataType::Structure(
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
            &CompoundDataTypeAST::Tuple(ref fields) => Ok(CompoundDataType::Tuple(Arc::new(fields.resolve(ctx)?))),
        }
    }
}

impl<'source> Resolve<SyncRef<Module>> for Vec<(Identifier<'source>, FieldAST<'source>)> {
    type Result = Vec<(String, Field)>;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        self.iter()
            .map(|&(ref name, ref field)| {
                let field = field.resolve(ctx)?;
                Ok((name.text().to_string(), field))
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataTypeAST<'source> {
    Compound(CompoundDataTypeAST<'source>),
    Primitive(PrimitiveDataType),
    Reference(ItemPath),
}

impl<'source> Assertion for DataTypeAST<'source> {
    fn assert(&self, other_data_type: &DataTypeAST) {
        match self {
            &DataTypeAST::Compound(ref compound_type) => {
                match_it!(other_data_type, &DataTypeAST::Compound(ref other_compound_type) => {
                    compound_type.assert(other_compound_type);
                });
            }
            &DataTypeAST::Reference(ref path) => {
                match_it!(other_data_type, &DataTypeAST::Reference(ref other_path) => {
                    assert_eq!(path.path, other_path.path);
                });
            }
            other => assert_eq!(other, other_data_type),
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

//impl SemanticResolve for DataType<'static> {
//    fn is_resolved(&self, context: &ResolveContext) -> bool {
//        match self {
//            &DataType::Compound(CompoundDataType::Structure(ref fields)) => {
//                fields.is_resolved(context)
//            }
//            &DataType::Compound(CompoundDataType::Tuple(ref fields)) => {
//                fields.is_resolved(context)
//            }
//            &DataType::Primitive(_) => true,
//            &DataType::Reference(_) => false,
//            &DataType::ItemReference(ref item) => item.0.read().is_resolved(context),
//        }
//    }
//    fn try_resolve(&mut self, context: &mut ResolveContext) {
//        let mut new_value = None;
//        match self {
//            &mut DataType::Compound(CompoundDataType::Structure(ref mut fields)) => {
//                fields.try_resolve(context);
//            }
//            &mut DataType::Compound(CompoundDataType::Tuple(ref mut fields)) => {
//                fields.try_resolve(context);
//            }
//            &mut DataType::Primitive(_) => {}
//            &mut DataType::Reference(ref path) => {
//                match context.resolve_item(&path) {
//                    Ok(dep_ref) => {
//                        match dep_ref.assert_type(ItemType::DataType, path.pos) {
//                            Ok(_) => new_value = Some(DataType::ItemReference(dep_ref)),
//                            Err(err) => context.throw_error(err),
//                        }
//                    }
//                    Err(err) => context.throw_error(err),
//                }
//            }
//            &mut DataType::ItemReference(_) => {
//                // TODO Item should be DataType
//            }
//        }
//        if let Some(new_value) = new_value {
//            replace(self, new_value);
//        }
//    }
//}
//
//impl SemanticResolve for Vec<Field<'static>> {
//    fn is_resolved(&self, context: &ResolveContext) -> bool {
//        self.iter()
//            .all(|item| item.field_type.is_resolved(context))
//    }
//    fn try_resolve(&mut self, context: &mut ResolveContext) {
//        for field in self.iter_mut() {
//            field.field_type.try_resolve(context);
//        }
//    }
//}
//
//impl SemanticResolve for Vec<(Identifier<'static>, Field<'static>)> {
//    fn is_resolved(&self, context: &ResolveContext) -> bool {
//        self.iter()
//            .all(|item| item.1.field_type.is_resolved(context))
//    }
//    fn try_resolve(&mut self, context: &mut ResolveContext) {
//        // Имена полей структуры должны быть уникальными
//        for (i, &(ref field_name, ref field)) in self.iter().enumerate() {
//            for &(ref field_before_name, _) in self[..i].iter() {
//                if field_before_name == field_name {
//                    context.throw_error(SemanticError::duplicate_definition(
//                        field.position,
//                        (*field_before_name).clone(),
//                        SemanticItemType::Field,
//                    ));
//                }
//            }
//        }
//        for &mut (_, ref mut field) in self.iter_mut() {
//            field.field_type.try_resolve(context);
//        }
//    }
//}

//impl<'source> IntoStatic for DataType<'source> {
//    type Result = DataType<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            DataType::Compound(data_type) => DataType::Compound(data_type.into_static()),
//            DataType::Primitive(data_type) => DataType::Primitive(data_type),
//            DataType::Reference(path) => DataType::Reference(path.into_static()),
//            DataType::ItemReference(refer) => DataType::ItemReference(refer),
//        }
//    }
//}

impl<'source> Resolve<SyncRef<Module>> for DataTypeAST<'source> {
    type Result = DataType;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<Module>) -> Result<Self::Result, Vec<Self::Error>> {
        match self {
            &DataTypeAST::Compound(ref value) => Ok(DataType::Compound(value.resolve(ctx)?)),
            &DataTypeAST::Primitive(ref value) => Ok(DataType::Primitive(value.clone())),
            &DataTypeAST::Reference(ref path) => {
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

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    Array(Arc<DataType>),
    Compound(CompoundDataType),
    Primitive(PrimitiveDataType),
    Reference(SyncRef<Item>),
    Void,
}

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
        if let &DataType::Reference(ref reference) = target {
            let guard = reference.read();
            let data_type = match guard.get_data_type() {
                Some(data_type) => data_type,
                None => return false,
            };
            return self.can_cast(&data_type.body);
        }
        match self {
            &DataType::Array(ref self_subtype) => {
                if let &DataType::Array(ref subtype) = target {
                    return self_subtype.can_cast(&*subtype);
                }
            }
            &DataType::Compound(ref self_subtype) => {
                if let &DataType::Compound(ref subtype) = target {
                    return self_subtype.can_cast(&*subtype);
                }
            }
            &DataType::Primitive(ref self_subtype) => {
                if let &DataType::Primitive(ref subtype) = target {
                    return self_subtype.can_cast(&*subtype);
                }
            }
            &DataType::Reference(ref reference) => {
                let guard = reference.read();
                let data_type = match guard.get_data_type() {
                    Some(data_type) => data_type,
                    None => return false,
                };
                return data_type.body.can_cast(target);
            }
            &DataType::Void => return *target == DataType::Void,
        }
        false
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DataType::Array(ref subtype) => write!(f, "[{}]", subtype),
            &DataType::Compound(CompoundDataType::Structure(ref fields)) => {
                let mut d = f.debug_struct("");
                for (name, field) in fields.iter() {
                    d.field(&name, &field.field_type);
                }
                d.finish()
            }
            &DataType::Compound(CompoundDataType::Tuple(ref components)) => {
                let mut d = f.debug_tuple("");
                for component in components.iter() {
                    d.field(&component.field_type);
                }
                d.finish()
            }
            &DataType::Primitive(ref primitive) => write!(f, "{}", primitive),
            &DataType::Reference(ref refer) => {
                let reference = refer.read();
                match reference.get_data_type() {
                    Some(def) => write!(f, "{}", def.body),
                    None => write!(f, "<not a type>"),
                }
            }
            &DataType::Void => write!(f, "!"),
        }
    }
}
