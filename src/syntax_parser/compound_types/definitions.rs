use helpers::assertion::Assertion;
use helpers::group::Group;
use lexeme_scanner::ItemPosition;
use syntax_parser::primitive_types::PrimitiveDataType;
use project_analysis::{
    DependencyReference,
    SemanticResolve,
    SemanticContext,
    SemanticError,
    SemanticItemType,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attribute<'source> {
    pub name: &'source str,
    pub arguments: Option<Vec<&'source str>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field<'source> {
    pub attributes: Vec<Attribute<'source>>,
    pub field_type: DataType<'source>,
    pub position: ItemPosition,
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
    DependencyReference(DependencyReference),
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

impl<'source> SemanticResolve for DataType<'source> {
    fn is_resolved(&self) -> bool {
        unimplemented!()
    }
    fn resolve(&mut self, _context: &mut SemanticContext) -> Result<(), Group<SemanticError>> {
        let mut errors = Group::None;
        match self {
            &mut DataType::Compound(CompoundDataType::Structure(ref fields)) => {
                for (i, &(field_name, ref field)) in fields.iter().enumerate() {
                    // Поля структуры должны быть уникальными
                    for &(field_before_name, _) in fields[..i].iter() {
                        if field_before_name == field_name {
                            errors.append_group(Group::One(SemanticError::DuplicateDefinition {
                                name: field_before_name,
                                pos: field.position,
                                item_type: SemanticItemType::Field,
                            }));
                        }
                    }
                }
                unimplemented!()
            },
            _ => unimplemented!(),
        }
    }
}
