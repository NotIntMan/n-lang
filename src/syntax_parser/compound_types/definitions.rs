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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompoundDataType<'source> {
    Structure(Vec<(Identifier<'source>, Field<'source>)>),
    Tuple(Vec<Field<'source>>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataType<'source> {
    Compound(CompoundDataType<'source>),
    Primitive(PrimitiveDataType),
    Reference(Vec<Identifier<'source>>),
    DependencyReference(DependencyReference),
}

impl<'source> Assertion<str> for DataType<'source> {
    fn assert(&self, other: &str) {
        let tokens = ::lexeme_scanner::Scanner::scan(other)
            .expect("Scanner result must be ok");
        let other_data_type = ::parser_basics::parse(
            tokens.as_slice(),
            ::syntax_parser::compound_types::data_type,
        ).expect("Parser result must be ok");
        assert_eq!(*self, other_data_type);
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
