use std::mem::replace;
use helpers::assertion::Assertion;
use helpers::into_static::IntoStatic;
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use syntax_parser::others::Path;
use syntax_parser::primitive_types::PrimitiveDataType;
use project_analysis::resolve::{
    SemanticResolve,
    ResolveContext,
};
use project_analysis::error::SemanticError;
use project_analysis::item::{
    ItemRef,
    ItemType,
    SemanticItemType,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attribute<'source> {
    pub name: Identifier<'source>,
    pub arguments: Option<Vec<Identifier<'source>>>,
}

impl<'source> IntoStatic for Attribute<'source> {
    type Result = Attribute<'static>;
    fn into_static(self) -> Self::Result {
        let Attribute { name, arguments } = self;
        Attribute {
            name: name.into_static(),
            arguments: arguments.into_static(),
        }
    }
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

impl<'source> IntoStatic for Field<'source> {
    type Result = Field<'static>;
    fn into_static(self) -> Self::Result {
        let Field {
            attributes,
            field_type,
            position,
        } = self;
        Field {
            attributes: attributes.into_static(),
            field_type: field_type.into_static(),
            position,
        }
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

impl<'source> IntoStatic for CompoundDataType<'source> {
    type Result = CompoundDataType<'static>;
    fn into_static(self) -> Self::Result {
        match self {
            CompoundDataType::Structure(fields) => CompoundDataType::Structure(fields.into_static()),
            CompoundDataType::Tuple(fields) => CompoundDataType::Tuple(fields.into_static()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataType<'source> {
    Compound(CompoundDataType<'source>),
    Primitive(PrimitiveDataType),
    Reference(Path<'source>),
    ItemReference(ItemRef),
}

impl<'source> Assertion for DataType<'source> {
    fn assert(&self, other_data_type: &DataType) {
        match self {
            &DataType::Compound(ref compound_type) => {
                match_it!(other_data_type, &DataType::Compound(ref other_compound_type) => {
                    compound_type.assert(other_compound_type);
                });
            }
            &DataType::Reference(ref path) => {
                match_it!(other_data_type, &DataType::Reference(ref other_path) => {
                    assert_eq!(path.path, other_path.path);
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

impl SemanticResolve for DataType<'static> {
    fn is_resolved(&self, context: &ResolveContext) -> bool {
        match self {
            &DataType::Compound(CompoundDataType::Structure(ref fields)) => {
                fields.iter()
                    .all(|item| item.1.field_type.is_resolved(context))
            }
            &DataType::Compound(CompoundDataType::Tuple(ref fields)) => {
                fields.iter()
                    .all(|item| item.field_type.is_resolved(context))
            }
            &DataType::Primitive(_) => true,
            &DataType::Reference(_) => false,
            &DataType::ItemReference(ref item) => item.0.read().is_resolved(context),
        }
    }
    fn try_resolve(&mut self, context: &mut ResolveContext) {
        let mut new_value = None;
        match self {
            &mut DataType::Compound(CompoundDataType::Structure(ref mut fields)) => {
                // Имена полей структуры должны быть уникальными
                for (i, &(ref field_name, ref field)) in fields.iter().enumerate() {
                    for &(ref field_before_name, _) in fields[..i].iter() {
                        if field_before_name == field_name {
                            context.throw_error(SemanticError::duplicate_definition(
                                field.position,
                                (*field_before_name).clone(),
                                SemanticItemType::Field,
                            ));
                        }
                    }
                }
                for &mut (_, ref mut field) in fields.iter_mut() {
                    field.field_type.try_resolve(context);
                }
            }
            &mut DataType::Compound(CompoundDataType::Tuple(ref mut fields)) => {
                for field in fields.iter_mut() {
                    field.field_type.try_resolve(context);
                }
            }
            &mut DataType::Primitive(_) => {}
            &mut DataType::Reference(ref path) => {
                if let Some(dep_ref) = context.resolve_item(ItemType::DataType, &path) {
                    new_value = Some(DataType::ItemReference(dep_ref))
                }
            }
            &mut DataType::ItemReference(_) => {
                // TODO Item should be DataType
            }
        }
        if let Some(new_value) = new_value {
            replace(self, new_value);
        }
    }
}

impl<'source> IntoStatic for DataType<'source> {
    type Result = DataType<'static>;
    fn into_static(self) -> Self::Result {
        match self {
            DataType::Compound(data_type) => DataType::Compound(data_type.into_static()),
            DataType::Primitive(data_type) => DataType::Primitive(data_type),
            DataType::Reference(path) => DataType::Reference(path.into_static()),
            DataType::ItemReference(refer) => DataType::ItemReference(refer),
        }
    }
}
