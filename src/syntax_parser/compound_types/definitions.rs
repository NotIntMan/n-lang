#![allow(unused_imports)]

use std::mem::replace;
use indexmap::IndexMap;
use helpers::assertion::Assertion;
//use helpers::into_static::IntoStatic;
use helpers::resolve::Resolve;
use helpers::sync_ref::SyncRef;
use helpers::as_unique::as_unique;
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use syntax_parser::others::Path;
use syntax_parser::primitive_types::PrimitiveDataType;
//use project_analysis::resolve::{
//    SemanticResolve,
//ResolveContext,
//};
//use project_analysis::error::SemanticError;
use project_analysis::item::{
    Item,
//    ItemRef,
//    ItemType,
//    SemanticItemType,
};
use project_analysis::module_context::ModuleContext;

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

pub fn find_attribute<'a>(attributes: &'a [Attribute], name: &str) -> Option<&'a Attribute> {
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

impl<'source> Resolve<ModuleContext> for FieldAST<'source> {
    type Result = Field;
    type Error = ();
    fn resolve(&mut self, ctx: &mut ModuleContext) -> Result<Self::Result, Self::Error> {
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CompoundDataType {
    Structure(IndexMap<String, Field>),
    Tuple(Vec<Field>),
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

impl<'source> Resolve<ModuleContext> for CompoundDataTypeAST<'source> {
    type Result = CompoundDataType;
    type Error = ();
    fn resolve(&mut self, ctx: &mut ModuleContext) -> Result<Self::Result, Self::Error> {
        match self {
            &mut CompoundDataTypeAST::Structure(ref mut fields) => Ok(CompoundDataType::Structure(
                as_unique(fields.resolve(ctx)?)?
            )),
            &mut CompoundDataTypeAST::Tuple(ref mut fields) => Ok(CompoundDataType::Tuple(fields.resolve(ctx)?)),
        }
    }
}

impl<'source> Resolve<ModuleContext> for Vec<(Identifier<'source>, FieldAST<'source>)> {
    type Result = Vec<(String, Field)>;
    type Error = ();
    fn resolve(&mut self, ctx: &mut ModuleContext) -> Result<Self::Result, Self::Error> {
        self.iter_mut()
            .map(|&mut (ref name, ref mut field)| {
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
    Reference(Path<'source>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataType {
    Compound(CompoundDataType),
    Primitive(PrimitiveDataType),
    Reference(SyncRef<Item>),
}

impl<'source> DataTypeAST<'source> {
//    pub fn prop(&self, _pos: ItemPosition, prop: &Identifier<'source>) -> Result<&DataType<'source>, SemanticError> {
//        match self {
//            &DataType::Compound(CompoundDataType::Structure(ref fields)) => {
//                if let Some(x) = fields.iter()
//                    .find(|&&(ref name, _)| name == prop)
//                    .map(|&(_, ref field)| &field.field_type)
//                    {
//                        return Ok(x);
//                    }
//            }
//            &DataType::Compound(CompoundDataType::Tuple(ref fields)) => {
//                // TODO Возможно, доступ к полям кортежа сейчас невозможен из-за того, что идентификатор не может начинаться с цифры.
//                if let Ok(index) = prop.get_text().parse::<usize>() {
//                    if let Some(x) = fields.get(index)
//                        .map(|field: &Field| &field.field_type)
//                        {
//                            return Ok(x);
//                        }
//                }
//            }
//            _ => {}
//        }
//        unimplemented!()
////        Err(SemanticError::wrong_property(pos, prop.clone().into_static()))
//    }
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
            ::syntax_parser::compound_types::data_type,
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

impl<'source> Resolve<ModuleContext> for DataTypeAST<'source> {
    type Result = DataType;
    type Error = ();
    fn resolve(&mut self, ctx: &mut ModuleContext) -> Result<Self::Result, Self::Error> {
        match self {
            &mut DataTypeAST::Compound(ref mut value) => Ok(DataType::Compound(value.resolve(ctx)?)),
            &mut DataTypeAST::Primitive(ref mut value) => Ok(DataType::Primitive(value.clone())),
            &mut DataTypeAST::Reference(ref mut path) => {
                let name = path.path
                    .first()
                    .unwrap()
                    .text();
                let item = ctx.get_item(name)
                    .ok_or(())?;
                // TODO Assert item.type == data_type
                Ok(DataType::Reference(item))
            }
        }
    }
}
