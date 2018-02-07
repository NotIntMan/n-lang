use std::collections::HashMap;

use desc_lang::primitives::PrimitiveDataType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub arguments: Option<Vec<&'a str>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field<'a> {
    pub attributes: Option<Vec<Attribute<'a>>>,
    pub field_type: DataType<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructureDataType<'a> {
    pub attributes: Option<Vec<Attribute<'a>>>,
    pub fields: HashMap<&'a str, Field<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TupleDataType<'a> {
    pub attributes: Option<Vec<Attribute<'a>>>,
    pub fields: Vec<Field<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataType<'a> {
    Primitive(PrimitiveDataType),
    Structure(StructureDataType<'a>),
    Tuple(TupleDataType<'a>),
    Reference(&'a str),
}
