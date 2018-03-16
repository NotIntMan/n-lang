use helpers::assertion::Assertion;

use syntax_parser::primitive_types::PrimitiveDataType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attribute<'source> {
    pub name: &'source str,
    pub arguments: Option<Vec<&'source str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticAttribute {
    pub name: String,
    pub arguments: Option<Vec<String>>,
}

derive_convert!(Attribute<'source> => SemanticAttribute {
    name,
    arguments is mappable iterable value,
});

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field<'source> {
    pub attributes: Vec<Attribute<'source>>,
    pub field_type: DataType<'source>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompoundDataType<'source> {
    Structure(Vec<(&'source str, Field<'source>)>),
    Tuple(Vec<Field<'source>>),
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
            ::syntax_parser::compound_types::data_type
        ).expect("Parser result must be ok");
        assert_eq!(*self, other_data_type);
    }
}

impl<'a, 'source> Assertion<&'a str> for DataType<'source> {
    fn assert(&self, other: &&'a str) {
        self.assert(*other)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SemanticDataType {
//    Compound(CompoundDataType<'source>),
    Primitive(PrimitiveDataType),
    Reference(Vec<String>),
}
