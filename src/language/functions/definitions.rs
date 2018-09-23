use helpers::{
    BlockFormatter,
    CodeFormatter,
    Extractor,
    Generate,
    generate_name,
    NameUniquer,
    Path,
    PathBuf,
    Resolve,
    SimpleFormatter,
    SyncRef,
    TSQL,
    TSQLParameters,
};
use indexmap::IndexMap;
use language::{
    AttributeAST,
    DataType,
    DataTypeAST,
    Expression,
    FieldPrimitive,
    find_attribute_ast,
    Statement,
    StatementAST,
    TableDefinition,
};
use lexeme_scanner::ItemPosition;
use parser_basics::Identifier;
use project_analysis::{
    FunctionContext,
    FunctionVariable,
    FunctionVariableScope,
    Item,
    Module,
    SemanticError,
    SemanticItemType,
    StatementFlowControlJumping,
    StatementFlowControlPosition,
};
use std::fmt::{
    self,
    Write,
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
    pub pos: ItemPosition,
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

        let (result_pos, result) = match &self.result {
            Some(data_type) => (data_type.pos, data_type.resolve(&ctx.0)?),
            None => (self.pos, DataType::Void),
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

        if let Some(sub_type) = result.as_array() {
            if !is_lite_weight {
                return SemanticError::not_allowed_inside(result_pos, "array type", "function with side effects")
                    .into_err_vec();
            }
            if sub_type.as_array().is_some() {
                // Unreachable but mean
                return SemanticError::not_allowed_inside(result_pos, "array type", "array type")
                    .into_err_vec();
            }
            if sub_type.as_primitive().is_some() {
                return SemanticError::not_allowed_inside(result_pos, "primitive type", "array type")
                    .into_err_vec();
            }
        }
        if !is_lite_weight && result.as_array().is_some() {
            return SemanticError::not_allowed_inside(result_pos, "array type", "function with side effects")
                .into_err_vec();
        }

        let result_var_name = if is_lite_weight && result.as_primitive().is_some() {
            None
        } else {
            Some(generate_name(
                |new_name| !arguments.contains_key(new_name),
                "return_value".to_string(),
            ))
        };

        Ok(FunctionDefinition {
            name: self.name.to_string(),
            arguments,
            result,
            result_var_name,
            body,
            context,
            is_lite_weight,
            pos: self.pos,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub arguments: IndexMap<String, SyncRef<FunctionVariable>>,
    pub result: DataType,
    pub result_var_name: Option<String>,
    pub body: FunctionBody,
    pub context: SyncRef<FunctionContext>,
    pub is_lite_weight: bool,
    pub pos: ItemPosition,
}

impl FunctionDefinition {
    pub fn fmt_primitives_as_args(
        mut f: BlockFormatter<impl fmt::Write>,
        primitives: Vec<FieldPrimitive>,
        context: &mut TSQLFunctionContext,
        is_automatic: bool,
        last_comma: bool,
        is_output: bool,
    ) -> fmt::Result {
        let mut arguments = primitives.into_iter().peekable();
        while let Some(primitive) = arguments.next() {
            let mut line = f.line()?;
            if !is_automatic {
                write!(line, "@{}", primitive.path)?;
            } else {
                write!(line, "[{}]", primitive.path)?;
            }
            write!(line, " {}", TSQL(&primitive.field_type, context.parameters.clone()))?;
            if is_output {
                line.write(" OUTPUT")?;
            }
            if last_comma || arguments.peek().is_some() {
                line.write(", ")?;
            }
        }
        Ok(())
    }
    pub fn fmt_arguments(
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        let is_procedure = !context.function.is_lite_weight;
        if !is_procedure {
            f.write_line('(')?;
        }
        let mut sub_f = f.sub_block();
        {
            let mut arguments = context.function.arguments.iter().peekable();
            while let Some((argument_name, argument)) = arguments.next() {
                let argument_guard = argument.read();
                let mut prefix = PathBuf::new("#");
                prefix.push(argument_name.as_str());
                let primitives = argument_guard.data_type()
                    .expect("Function argument cannot have unknown data-type")
                    .primitives(prefix);
                FunctionDefinition::fmt_primitives_as_args(
                    sub_f.clone(),
                    primitives,
                    context,
                    false,
                    is_procedure || arguments.peek().is_some(),
                    false,
                )?;
            }
        }
        if is_procedure {
            let table = if context.function.result.can_be_table() {
                context.function.result.as_table_type(context.make_result_variable_prefix())
            } else {
                None
            };
            if let Some(primitives) = table {
                FunctionDefinition::fmt_primitives_as_args(
                    sub_f,
                    primitives,
                    context,
                    false,
                    false,
                    true,
                )?;
            } else {
                if let Some(result_variable_name) = &context.function.result_var_name {
                    let mut line = sub_f.line()?;
                    line.write(format_args!("@{} ", result_variable_name))?;
                    if let Some(result) = context.function.result.as_primitive() {
                        line.write(TSQL(&result, context.parameters.clone()))?;
                    } else {
                        line.write("bit")?;
                    }
                    line.write(" OUTPUT")?;
                }
            }
        } else {
            let table = if context.function.result.can_be_table() {
                context.function.result.as_table_type(PathBuf::new("#"))
            } else {
                None
            };
            if let Some(primitives) = table {
                let result_variable_name = context.function.result_var_name.as_ref()
                    .expect("Table-valued functions should know their return-value name.");
                f.write_line(format_args!(") RETURNS @{} TABLE (", result_variable_name))?;
                FunctionDefinition::fmt_primitives_as_args(
                    sub_f,
                    primitives,
                    context,
                    true,
                    false,
                    false,
                )?;
                f.write_line(')')?;
            } else {
                if let Some(result) = context.function.result.as_primitive() {
                    f.write_line(format_args!(") RETURNS {}", TSQL(&result, context.parameters.clone())))?;
                } else {
                    f.write_line(") RETURNS bit")?;
                }
            }
        }
        Ok(())
    }
    pub fn fmt_head(
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        let sub_f = f.sub_block();
        // TODO Добавить переменную-результат в контекст (в случае табличных данных на выходе)
        let class = if context.function.is_lite_weight { "FUNCTION" } else { "PROCEDURE" };
        f.write_line(format_args!("CREATE OR ALTER {} dbo.[{}]", class, context.make_function_name().data))?;
        FunctionDefinition::fmt_arguments(sub_f.clone(), context)
    }
    pub fn fmt_variable(
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
        var: &FunctionVariable,
    ) -> fmt::Result {
        if var.is_automatic() || var.is_argument() { return Ok(()); }
        // TODO Адекватный проброс ошибок наверх
        let data_type = var.data_type()
            .expect("Variable must have determined data-type in generate-time");
        if let DataType::Array(sub_type) = data_type {
            f.write_line(format_args!("DECLARE @{} TABLE (", var.name()))?;
            TableDefinition::fmt_primitives_as_columns(
                f.sub_block(),
                context.parameters.clone(),
                sub_type.primitives(PathBuf::new("#")),
                false,
                None,
            )?;
            f.write_line(");")?;
        } else {
            let mut prefix = PathBuf::new("#");
            prefix.push(var.name());
            for primitive in data_type.primitives(prefix) {
                f.write_line(format_args!("DECLARE @{} {};", primitive.path, TSQL(&primitive.field_type, context.parameters.clone())))?;
            }
        }
        Ok(())
    }
    pub fn fmt_body(
        mut f: BlockFormatter<impl fmt::Write>,
        context: &mut TSQLFunctionContext,
    ) -> fmt::Result {
        let body = match &context.function.body {
            FunctionBody::Implementation(stmt) => stmt,
            FunctionBody::External => return Ok(()),
        };

        f.write_line("AS BEGIN")?;
        let mut sub_f = f.sub_block();

        for variable in context.function.context.get_all_variables() {
            if variable.is_automatic() || variable.is_argument() { continue; }
            let mut variable_guard = variable.write();
            let new_name = context.names.add_name(variable_guard.name().into());
            variable_guard.set_name(new_name);
            FunctionDefinition::fmt_variable(sub_f.clone(), context, &*variable_guard)?;
        }

        {
            let array;
            let statements = if let Some(statements) = body.as_block() {
                statements
            } else {
                array = [body.clone()];
                &array[..]
            };
            Statement::fmt_block_without_parens(
                sub_f.clone(),
                context,
                statements,
            )?;
        }

        if context.function.result == DataType::Void {
            if let Some(var_name) = &context.function.result_var_name {
                sub_f.write_line(format_args!("SET @{} = 0;", var_name))?;
            }
        }
        if context.function.is_lite_weight {
            if context.function.result.as_primitive().is_some() {
                sub_f.write_line("RETURN 0;")?;
            } else {
                sub_f.write_line("RETURN;")?;
            }
        }

        f.write_line("END")
    }
    pub fn fmt_export(
        &self,
        f: &mut SimpleFormatter,
        module_path: Path,
    ) -> fmt::Result {
        writeln!(f, "export function {}(", self.name)?;

        // Arguments
        {
            let mut args_f = f.sub_block();
            writeln!(args_f, "_pool: _mssql.ConnectionPool,")?;
            for (argument_name, argument) in &self.arguments {
                write!(args_f, "{}: ", argument_name)?;
                let argument_guard = argument.read();
                let argument_data_type = argument_guard.data_type()
                    .expect("Variable's data-type should not be unknown at generate-time");
                argument_data_type.fmt(&mut args_f.sub_block())?;
                writeln!(args_f, ",")?;
            }
        }

        // Return type
        write!(f, "): Promise<")?;
        self.result.fmt(&mut f.sub_block())?;
        writeln!(f, "> {{")?;

        // Body
        {
            let mut body_f = f.sub_block();

            // Binding arguments into the query
            writeln!(body_f, "const _req = new _mssql.Request(_pool)")?;
            for (argument_name, argument) in &self.arguments {
                let argument_guard = argument.read();
                let argument_data_type = argument_guard.data_type()
                    .expect("Variable's data-type should not be unknown at generate-time");
                let mut prefix = PathBuf::new("#");
                prefix.push(&*argument_name);
                for primitive in argument_data_type.primitives(prefix) {
                    let data_path = primitive.path.as_path().into_new_buf(".");
                    write!(body_f, "_req.input('{target}', _mssql.", target = primitive.path)?;
                    primitive.field_type.fmt_ts_mssql(&mut body_f)?;
                    writeln!(body_f, ", {source})", source = data_path)?;
                }
            }


            if self.is_lite_weight {
                // Arguments of function for call expression
                let arguments = {
                    let mut buffer = String::new();
                    let mut arguments = self.arguments.iter()
                        .peekable();
                    while let Some((argument_name, argument)) = arguments.next() {
                        let argument_guard = argument.read();
                        let argument_data_type = argument_guard.data_type()
                            .expect("Variable's data-type should not be unknown at generate-time");

                        let mut prefix = PathBuf::new("#");
                        prefix.push(&*argument_name);
                        let mut primitives = argument_data_type.primitives(prefix)
                            .into_iter()
                            .peekable();
                        while let Some(primitive) = primitives.next() {
                            buffer.push('@');
                            buffer.push_str(primitive.path.data.as_str());
                            if primitives.peek().is_some() || arguments.peek().is_some() {
                                buffer.push_str(", ");
                            }
                        }
                    }
                    buffer
                };

                // For primitive results
                if self.result.as_primitive().is_some() {
                    writeln!(
                        body_f,
                        "return _req.query('SELECT dbo.[{module}::{name}]({args}) as result')",
                        module = module_path,
                        name = self.name,
                        args = arguments,
                    )?;
                    let mut then_f = body_f.sub_block();
                    writeln!(then_f, ".then(_result => {{")?;
                    {
                        let mut closure_f = then_f.sub_block();
                        writeln!(closure_f, "return _result.recordset[0].result")?;
                    }
                    writeln!(then_f, "}})")?;
                } else {
                    writeln!(
                        body_f,
                        "return _req.query('SELECT * FROM dbo.[{module}::{name}]({args})')",
                        module = module_path,
                        name = self.name,
                        args = arguments,
                    )?;

                    // Extracting result
                    let mut then_f = body_f.sub_block();
                    writeln!(then_f, ".then(_result => {{")?;
                    {
                        let mut closure_f = then_f.sub_block();
                        let (is_array, result) = self.result.as_array()
                            .map(|t| (true, &**t))
                            .unwrap_or_else(|| (false, &self.result));
                        writeln!(closure_f, "return _result.recordset.map(record => {{")?;
                        {
                            // Binding results
                            let mut sub_closure_f = closure_f.sub_block();
                            sub_closure_f.write_str("return ")?;
                            result.fmt_result_bind(
                                &mut sub_closure_f,
                                "record",
                                Path::new("", "#"),
                            )?;
                            writeln!(sub_closure_f, "")?;
                        }

                        // Recordset is always an array
                        // but if needs a one item, not an array
                        // then we extract the first item and return it
                        writeln!(closure_f, "}}){}", if is_array {
                            ""
                        } else {
                            "[0]"
                        })?;
                    }
                    writeln!(then_f, "}})")?;
                }
            } else {
                // Binding result of procedure
                let mut prefix = PathBuf::new("#");
                let result_variable_name = self.result_var_name.as_ref()
                    .expect("Procedures should know their return-value name.");
                prefix.push(result_variable_name);
                for primitive in self.result.primitives(prefix) {
                    write!(body_f, "_req.output('{}', _mssql.", primitive.path)?;
                    primitive.field_type.fmt_ts_mssql(&mut body_f)?;
                    writeln!(body_f, ")")?;
                }

                // Calling procedure
                writeln!(
                    body_f,
                    "return _req.execute('dbo.[{module}::{name}]')",
                    module = module_path,
                    name = self.name,
                )?;

                // Awaiting result
                let mut then_f = body_f.sub_block();
                writeln!(then_f, ".then(_result => {{")?;

                {
                    // Binding result
                    let mut closure_f = then_f.sub_block();
                    write!(closure_f, "return ")?;
                    self.result.fmt_result_bind(
                        &mut closure_f,
                        "_result.output",
                        Path::new(result_variable_name, "#"),
                    )?;
                    writeln!(closure_f, "")?;
                }

                writeln!(then_f, "}})")?;
            }
        }

        /// End of function
        writeln!(f, "}}")
    }
}

impl<'a> Generate<TSQLParameters<'a>> for FunctionDefinition {
    fn fmt(&self, f: BlockFormatter<impl fmt::Write>, parameters: TSQLParameters<'a>) -> fmt::Result {
        let mut context = TSQLFunctionContext::new(self, parameters);
        FunctionDefinition::fmt_head(f.clone(), &mut context)?;
        FunctionDefinition::fmt_body(f, &mut context)
    }
}

#[derive(Debug, Clone)]
pub struct TSQLFunctionContext<'a, 'b> {
    pub function: &'a FunctionDefinition,
    pub parameters: TSQLParameters<'b>,
    pub names: NameUniquer,
    pub function_name: Option<PathBuf>,
    // TODO Учесть пре-вызовы перед каждой вставкой выражения
    pub temp_vars_scope: SyncRef<FunctionVariableScope>,
    pub pre_calc_calls: Vec<String>,
}

impl<'a, 'b> TSQLFunctionContext<'a, 'b> {
    pub fn new(function: &'a FunctionDefinition, parameters: TSQLParameters<'b>) -> Self {
        let temp_vars_scope = function.context.root().child();
        let mut names = NameUniquer::new();
        for (name, _) in &function.arguments {
            names.add_name(name.clone());
        }
        if let Some(name) = &function.result_var_name {
            names.add_name(name.clone());
        }
        Self {
            function,
            parameters,
            names,
            function_name: None,
            temp_vars_scope,
            pre_calc_calls: Vec::new(),
        }
    }
    pub fn make_function_name(&mut self) -> Path {
        if self.function_name.is_none() {
            let mut path = self.parameters.module_path.into_buf();
            path.push(self.function.name.as_str());
            self.function_name = Some(path)
        }
        match &self.function_name {
            Some(function_name) => return function_name.as_path(),
            None => unreachable!()
        }
    }
    pub fn make_result_variable_prefix(&mut self) -> PathBuf {
        let mut prefix = PathBuf::new("#");
        if let Some(result_var_name) = &self.function.result_var_name {
            prefix.push(&*result_var_name);
        }
        prefix
    }
    pub fn add_pre_calc_call(&mut self, function: &SyncRef<Item>, arguments: &[Expression]) -> Result<SyncRef<FunctionVariable>, fmt::Error> {
        let result_name = self.names.add_name("t".into());
        let result_data_type = {
            let function_guard = function.read();
            let inner_function = function_guard.get_function()
                .expect("Not-functions in function calls should not exist at generate-time");
            inner_function.result.clone()
        };
        let var = self.temp_vars_scope.new_variable(
            self.function.pos,
            result_name,
            Some(result_data_type),
        )
            .expect("Temp variable should not fail while initializing");
        var.make_read_only();
        let mut buffer = String::new();
        {
            let mut code_formatter = CodeFormatter::new(&mut buffer);
            code_formatter.indent_size = 4;
            let f = code_formatter.root_block();
            FunctionDefinition::fmt_variable(
                f.clone(),
                self,
                &*var.read(),
            )?;
            Statement::fmt_pre_call(
                f,
                Some(&var),
                function,
                arguments,
                self,
            )?;
        }
        self.pre_calc_calls.push(buffer);
        Ok(var)
    }
    #[inline]
    pub fn extract_pre_calc_calls(&mut self) -> Extractor<String> {
        Extractor::new(&mut self.pre_calc_calls)
    }
}
