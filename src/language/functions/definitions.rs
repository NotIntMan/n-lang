use indexmap::IndexMap;
//use helpers::IntoStatic;
use helpers::{
    as_unique_identifier,
    Resolve,
    SyncRef,
};
use parser_basics::Identifier;
use language::{
    AttributeAST,
    DataType,
    DataTypeAST,
    find_attribute,
    Statement,
    StatementAST,
};
use project_analysis::{
    FunctionContext,
    FunctionVariableScope,
    Module,
    SemanticError,
    SemanticItemType,
//    SemanticResolve,
//    ResolveContext,
};

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBodyAST<'source> {
    External,
    Implementation(StatementAST<'source>),
}

impl<'source> Resolve<SyncRef<FunctionVariableScope>> for FunctionBodyAST<'source> {
    type Result = FunctionBody;
    type Error = SemanticError;
    fn resolve(&self, ctx: &SyncRef<FunctionVariableScope>) -> Result<Self::Result, Vec<Self::Error>> {
        let result = match self {
            &FunctionBodyAST::External => FunctionBody::External,
            &FunctionBodyAST::Implementation(ref stmt) => FunctionBody::Implementation(stmt.resolve(ctx)?),
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    External,
    Implementation(Statement),
}

//impl<'source> IntoStatic for FunctionBody<'source> {
//    type Result = FunctionBody<'static>;
//    fn into_static(self) -> Self::Result {
//        match self {
//            FunctionBody::External => FunctionBody::External,
//            FunctionBody::Implementation(stmt) => FunctionBody::Implementation(stmt.into_static()),
//        }
//    }
//}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinitionAST<'source> {
    pub name: Identifier<'source>,
    pub arguments: Vec<(Identifier<'source>, DataTypeAST<'source>)>,
    pub result: Option<DataTypeAST<'source>>,
    pub body: FunctionBodyAST<'source>,
}

//impl<'source> IntoStatic for FunctionDefinition<'source> {
//    type Result = FunctionDefinition<'static>;
//    fn into_static(self) -> Self::Result {
//        let FunctionDefinition { name, arguments, result, body } = self;
//        FunctionDefinition {
//            name: name.into_static(),
//            arguments: arguments.into_static(),
//            result: result.into_static(),
//            body: body.into_static(),
//        }
//    }
//}

//impl SemanticResolve for FunctionDefinition<'static> {
//    fn is_resolved(&self, _context: &ResolveContext) -> bool {
//        unimplemented!()
//    }
//    fn try_resolve(&mut self, _context: &mut ResolveContext) {
//        unimplemented!()
//    }
//}

impl<'source> Resolve<(SyncRef<Module>, Vec<AttributeAST<'source>>)> for FunctionDefinitionAST<'source> {
    type Result = FunctionDefinition;
    type Error = SemanticError;
    fn resolve(&self, ctx: &(SyncRef<Module>, Vec<AttributeAST<'source>>)) -> Result<Self::Result, Vec<Self::Error>> {
        let arguments = match as_unique_identifier(self.arguments.clone()) {
            Ok(map) => map.resolve(&ctx.0)?,
            Err(name) => return Err(vec![SemanticError::duplicate_definition(
                name.item_pos(),
                name.text().to_string(),
                SemanticItemType::Variable,
            )])
        };
        let result = match &self.result {
            &Some(ref data_type) => data_type.resolve(&ctx.0)?,
            &None => DataType::Void,
        };

        let context = FunctionContext::new(ctx.0.clone());
        let root = context.root();
        let mut errors = Vec::new();

        for (name, data_type) in arguments.iter() {
            let name = name.as_str();
            let &(ident, _) = self.arguments.iter()
                .find(|&&(ref ident, _)| ident.text() == name)
                .expect("The argument has already been preprocessed and its name can not not exist in the input data");
            if let Err(error) = root.new_variable(ident.item_pos(), name.to_string(), Some(data_type.clone())) {
                errors.push(error);
            }
        }

        let body = self.body.resolve(&root)?;

        let has_side_effects = match &body {
            &FunctionBody::External => {
                find_attribute(&ctx.1, "no_side_effects").is_none()
            }
            &FunctionBody::Implementation(ref stmt) => {
                stmt.has_side_effects()
            }
        };

        Ok(FunctionDefinition {
            name: self.name.to_string(),
            arguments,
            result,
            body,
            context,
            has_side_effects,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub arguments: IndexMap<String, DataType>,
    pub result: DataType,
    pub body: FunctionBody,
    pub context: SyncRef<FunctionContext>,
    pub has_side_effects: bool,
}
