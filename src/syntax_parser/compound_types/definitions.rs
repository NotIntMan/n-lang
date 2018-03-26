//use std::mem::replace;
use helpers::assertion::Assertion;
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use syntax_parser::primitive_types::PrimitiveDataType;
use project_analysis::{
    DependencyReference,
//    SemanticResolve,
//    SemanticContext,
//    SemanticError,
//    SemanticItemType,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attribute<'source> {
    pub name: Identifier<'source>,
    pub arguments: Option<Vec<Identifier<'source>>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field<'source> {
    pub attributes: Vec<Attribute<'source>>,
    pub field_type: DataType<'source>,
    pub position: ItemPosition,
}

impl<'source> Assertion for Field<'source> {
    fn assert(&self, other: &Field) {
        assert_eq!(self.attributes, other.attributes);
        self.field_type.assert(&other.field_type);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompoundDataType<'source> {
    Structure(Vec<(Identifier<'source>, Field<'source>)>),
    Tuple(Vec<Field<'source>>),
}

impl<'source> Assertion for CompoundDataType<'source> {
    fn assert(&self, other: &CompoundDataType) {
        match self {
            &CompoundDataType::Structure(ref fields) => {
                let mut other_fields_iter = match_it!(other,
                    &CompoundDataType::Structure(ref fields) => { fields.iter() }
                );
                for &(ref field_name, ref field) in fields.iter() {
                    let &(ref other_field_name, ref other_field) = other_fields_iter.next()
                        .expect("Field lists should have equal sizes");
                    assert_eq!(field_name, other_field_name);
                    field.assert(other_field);
                }
                assert_eq!(other_fields_iter.next(), None);
            }
            &CompoundDataType::Tuple(ref fields) => {
                let mut other_fields_iter = match_it!(other,
                    &CompoundDataType::Tuple(ref fields) => { fields.iter() }
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataType<'source> {
    Compound(CompoundDataType<'source>),
    Primitive(PrimitiveDataType),
    Reference(Vec<Identifier<'source>>),
    DependencyReference(DependencyReference),
}

impl<'source> Assertion for DataType<'source> {
    fn assert(&self, other_data_type: &DataType) {
        match self {
            &DataType::Compound(ref compound_type) => {
                match_it!(other_data_type, &DataType::Compound(ref other_compound_type) => {
                    compound_type.assert(other_compound_type);
                });
            }
            other => assert_eq!(other, other_data_type),
        }
    }
}

impl<'source> Assertion<str> for DataType<'source> {
    fn assert(&self, other: &str) {
        let tokens = ::lexeme_scanner::Scanner::scan(other)
            .expect("Scanner result must be ok");
        let other_data_type = ::parser_basics::parse(
            tokens.as_slice(),
            ::syntax_parser::compound_types::data_type,
        ).expect("Parser result must be ok");
        self.assert(&other_data_type);
    }
}

impl<'a, 'source> Assertion<&'a str> for DataType<'source> {
    fn assert(&self, other: &&'a str) {
        self.assert(*other)
    }
}

//impl<'source> SemanticResolve for DataType<'source> {
//    fn is_resolved(&self) -> bool {
//        match self {
//            &DataType::Compound(CompoundDataType::Structure(ref fields)) => {
//                fields.iter()
//                    .all(|item| item.1.field_type.is_resolved())
//            }
//            &DataType::Compound(CompoundDataType::Tuple(ref fields)) => {
//                fields.iter()
//                    .all(|item| item.field_type.is_resolved())
//            }
//            &DataType::Primitive(_) => true,
//            &DataType::Reference(_) => false,
//            &DataType::DependencyReference(_) => true,
//        }
//    }
//    fn try_resolve(&mut self, context: &mut SemanticContext) {
//        let mut new_value = None;
//        match self {
//            &mut DataType::Compound(CompoundDataType::Structure(ref mut fields)) => {
//                for (i, &mut (field_name, ref mut field)) in fields.iter_mut().enumerate() {
//                    // Имена полей структуры должны быть уникальными
//                    for &(field_before_name, _) in fields[..i].iter() {
//                        if field_before_name == field_name {
//                            context.error(SemanticError::DuplicateDefinition {
//                                name: field_before_name,
//                                pos: field.position,
//                                item_type: SemanticItemType::Field,
//                            });
//                        }
//                    }
//                    field.field_type.try_resolve(context);
//                }
//            }
//            &mut DataType::Compound(CompoundDataType::Tuple(ref mut fields)) => {
//                for field in fields.iter_mut() {
//                    field.field_type.try_resolve(context);
//                }
//            }
//            &mut DataType::Primitive(_) => {}
//            &mut DataType::Reference(ref path) => {
//                match context.resolve(SemanticItemType::DataType, path.as_slice()) {
//                    Ok(dep_ref) => new_value = Some(DataType::DependencyReference(dep_ref)),
//                    Err(error) => context.error(error),
//                }
//            },
//            &mut DataType::DependencyReference(_) => {},
//        }
//        if let Some(new_value) = new_value {
//            replace(self, new_value);
//        }
//    }
//}
