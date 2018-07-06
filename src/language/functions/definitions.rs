use helpers::{
    Resolve,
    SyncRef,
};
use indexmap::IndexMap;
use language::{
    AttributeAST,
    DataType,
    DataTypeAST,
    find_attribute_ast,
    Statement,
    StatementAST,
};
use parser_basics::Identifier;
use project_analysis::{
    FunctionContext,
    FunctionVariable,
    FunctionVariableScope,
    Module,
    SemanticError,
    SemanticItemType,
    StatementFlowControlJumping,
    StatementFlowControlPosition,
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
            FunctionBodyAST::External => FunctionBody::External,
            FunctionBodyAST::Implementation(stmt) => FunctionBody::Implementation(stmt.resolve(ctx)?),
        };
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    External,
    Implementation(Statement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinitionAST<'source> {
    pub name: Identifier<'source>,
    pub arguments: Vec<(Identifier<'source>, DataTypeAST<'source>)>,
    pub result: Option<DataTypeAST<'source>>,
    pub body: FunctionBodyAST<'source>,
}

impl<'source> Resolve<(SyncRef<Module>, Vec<AttributeAST<'source>>)> for FunctionDefinitionAST<'source> {
    type Result = FunctionDefinition;
    type Error = SemanticError;
    fn resolve(&self, ctx: &(SyncRef<Module>, Vec<AttributeAST<'source>>)) -> Result<Self::Result, Vec<Self::Error>> {
        let context = FunctionContext::new(ctx.0.clone());
        let root = context.root();
        let mut errors = Vec::new();

        let mut arguments = IndexMap::new();
        for (identifier, data_type) in self.arguments.iter() {
            let name = identifier.text();
            let position = identifier.item_pos();
            if arguments.contains_key(name) {
                errors.push(SemanticError::duplicate_definition(
                    position,
                    name.to_string(),
                    SemanticItemType::Variable,
                ));
                continue;
            }
            let data_type = match data_type.resolve(&ctx.0) {
                Ok(data_type) => data_type,
                Err(mut sub_errors) => {
                    errors.append(&mut sub_errors);
                    continue;
                }
            };
            let var = match root.new_variable(position, name.to_string(), Some(data_type)) {
                Ok(var) => var,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };
            var.make_read_only();
            var.mark_as_argument();
            arguments.insert(name.to_string(), var);
        }

        let result = match &self.result {
            Some(data_type) => data_type.resolve(&ctx.0)?,
            None => DataType::Void,
        };

        let body = self.body.resolve(&root)?;

        if let FunctionBody::Implementation(body) = &body {
            let body_jumping = body.jumping_check(StatementFlowControlPosition::new(), &result)?;
            if (body_jumping != StatementFlowControlJumping::AlwaysReturns)
                && (result != DataType::Void) {
                return SemanticError::not_all_branches_returns(body.pos)
                    .into_err_vec();
            }
        }

        let is_lite_weight = match &body {
            FunctionBody::External => {
                find_attribute_ast(&ctx.1, "is_lite_weight").is_some()
            }
            FunctionBody::Implementation(stmt) => {
                stmt.is_lite_weight()
            }
        };

        Ok(FunctionDefinition {
            name: self.name.to_string(),
            arguments,
            result,
            body,
            context,
            is_lite_weight,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub arguments: IndexMap<String, SyncRef<FunctionVariable>>,
    pub result: DataType,
    pub body: FunctionBody,
    pub context: SyncRef<FunctionContext>,
    pub is_lite_weight: bool,
}
