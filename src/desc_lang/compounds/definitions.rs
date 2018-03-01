use helpers::assertion::Assertion;

use desc_lang::primitives::PrimitiveDataType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attribute<'source> {
    pub name: &'source str,
    pub arguments: Option<Vec<&'source str>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field<'source> {
    pub attributes: Vec<Attribute<'source>>,
    pub field_type: DataType<'source>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StructureDataType<'source> {
    pub attributes: Vec<Attribute<'source>>,
    pub fields: Vec<(&'source str, Field<'source>)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TupleDataType<'source> {
    pub attributes: Vec<Attribute<'source>>,
    pub fields: Vec<Field<'source>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompoundDataType<'source> {
    Structure(StructureDataType<'source>),
    Tuple(TupleDataType<'source>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataType<'source> {
    Compound(CompoundDataType<'source>),
    Primitive(PrimitiveDataType),
    Reference(Vec<&'source str>),
}

impl<'source> Assertion<str> for DataType<'source> {
    fn assert(&self, other: &str) {
        let tokens = ::lexeme_scanner::Scanner::scan(other)
            .expect("Scanner result must be ok");
        let other_data_type = ::parser_basics::parse(
            tokens.as_slice(),
            ::desc_lang::compounds::data_type
        ).expect("Parser result must be ok");
        assert_eq!(*self, other_data_type);
    }
}

impl<'a, 'source> Assertion<&'a str> for DataType<'source> {
    fn assert(&self, other: &&'a str) {
        self.assert(*other)
    }
}
