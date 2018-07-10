use helpers::{
    BlockFormatter,
    CodeFormatter,
    Generate,
    Map,
    PathBuf,
    SyncRef,
    TSQL,
    TSQLParameters,
};
use indexmap::IndexMap;
use language::{
    CompoundDataType,
    DataType,
    FunctionDefinition,
    TableDefinition,
};
use project_analysis::{
    Item,
    ItemBody,
    Module,
};
use std::{
    collections::HashMap,
    fmt::{
        self,
        Write,
    },
};

#[derive(Debug, Clone)]
pub struct DataClass {
    path: PathBuf,
    fields: Map<String, DataType>,
}

impl DataClass {
    pub fn for_data_type(path: PathBuf, reflection_target: &DataType, data_classes: &mut HashMap<PathBuf, Option<DataClass>>) {
        if data_classes.contains_key(&path) { return; }
        data_classes.insert(path.clone(), None);
        match reflection_target {
            DataType::Compound(CompoundDataType::Structure(fields)) => {
                let mut result_fields = Map::new();
                for (field_name, field) in fields.iter() {
                    let mut sub_path = path.clone();
                    sub_path.push(field_name.as_str());
                    DataClass::for_data_type(sub_path, &field.field_type, data_classes);
                    result_fields.insert(field_name.as_str(), field.field_type.clone());
                }
                data_classes.insert(path.clone(), Some(Self {
                    path,
                    fields: result_fields,
                }));
            }
            DataType::Compound(CompoundDataType::Tuple(fields)) => {
                let mut result_fields = Map::new();
                for (index, field) in fields.iter().enumerate() {
                    let field_name = format!("component{}", index);
                    let mut sub_path = path.clone();
                    sub_path.push(field_name.as_str());
                    DataClass::for_data_type(sub_path, &field.field_type, data_classes);
                    result_fields.insert(field_name, field.field_type.clone());
                }
                data_classes.insert(path.clone(), Some(Self {
                    path,
                    fields: result_fields,
                }));
            }
            DataType::Reference(item) => {
                DataClass::for_shared_item(item, data_classes);
            }
            _ => {}
        }
    }
    pub fn for_item(item: &Item, data_classes: &mut HashMap<PathBuf, Option<DataClass>>) {
        match item.body() {
            ItemBody::DataType { def } => {
                DataClass::for_data_type(item.get_path(), &def.body, data_classes);
            }
            ItemBody::Table { entity, primary_key, .. } => {
                DataClass::for_shared_item(entity, data_classes);
                DataClass::for_shared_item(primary_key, data_classes);
            }
            _ => {}
        }
    }
    #[inline]
    pub fn for_shared_item(item: &SyncRef<Item>, data_classes: &mut HashMap<PathBuf, Option<DataClass>>) {
        let item = item.read();
        DataClass::for_item(&*item, data_classes);
    }
}

#[derive(Debug, Clone)]
pub struct RPCModule {
    path: PathBuf,
    functions: Vec<FunctionDefinition>,
}

impl RPCModule {
    pub fn new(source: &Module, data_classes: &mut HashMap<PathBuf, Option<DataClass>>) -> Self {
        let module_path = source.path().read();
        let mut result = Self {
            path: module_path.clone(),
            functions: Vec::new(),
        };
        for (item_name, item) in source.items().iter() {
            let item = item.value.read();
            if let Some(function) = item.get_function() {
                let mut function = function.clone();
                function.name = item_name.clone();
                let mut function_path = module_path.clone();
                function_path.push(function.name.as_str());
                for (argument_name, argument) in function.arguments.iter() {
                    let mut sub_path = function_path.clone();
                    sub_path.push(argument_name.as_str());
                    let argument_guard = argument.read();
                    let argument_type = argument_guard.data_type()
                        .expect("Function arguments cannot have undefined data type");
                    DataClass::for_data_type(sub_path, argument_type, data_classes);
                }
                function_path.push("result");
                DataClass::for_data_type(function_path, &function.result, data_classes);
                result.functions.push(function);
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct RPCProject {
    rpc_modules: HashMap<PathBuf, RPCModule>,
    data_classes: HashMap<PathBuf, DataClass>,
}

impl RPCProject {
    pub fn new(project: &IndexMap<SyncRef<PathBuf>, SyncRef<Module>>) -> Self {
        let mut data_classes = HashMap::new();
        let mut rpc_modules = HashMap::new();

        for (module_path, module) in project.iter() {
            let module_guard = module.read();

            for (_, item) in module_guard.items().iter() {
                DataClass::for_shared_item(&item.value, &mut data_classes);
            }

            rpc_modules.insert(
                module_path.read().clone(),
                RPCModule::new(&*module_guard, &mut data_classes),
            );
        }
        let data_classes = data_classes.into_iter()
            .filter_map(|(path, data_class)|
                data_class.map(move |inner_data_class| (path, inner_data_class))
            )
            .collect::<HashMap<_, _>>();
        Self {
            data_classes,
            rpc_modules,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseModule {
    path: PathBuf,
    tables: Vec<TableDefinition>,
    functions: Vec<FunctionDefinition>,
}

impl DatabaseModule {
    pub fn new(source: &SyncRef<Module>) -> Self {
        let source_guard = source.read();
        let mut result = Self {
            path: source_guard.path().read().clone(),
            tables: Vec::new(),
            functions: Vec::new(),
        };
        for (item_name, item) in source_guard.items().iter() {
            let item = item.value.read();
            if !item.is_belongs_to(source) { continue; }
            if let Some(table) = item.get_table() {
                let mut table = table.clone();
                table.name = item_name.clone();
                result.tables.push(table);
                continue;
            }
            if let Some(function) = item.get_function() {
                let mut function = function.clone();
                function.name = item_name.clone();
                result.functions.push(function);
                continue;
            }
        }
        result
    }
    pub fn generate_tables(&self, mut f: BlockFormatter<impl Write>) -> fmt::Result {
        let parameters = TSQLParameters::new(self.path.as_path());
        for table in self.tables.iter() {
            Generate::fmt(table, f.clone(), parameters.clone())?;
            f.write_line("GO")?;
        }
        Ok(())
    }
    pub fn generate_functions(&self, mut f: BlockFormatter<impl Write>) -> fmt::Result {
        let parameters = TSQLParameters::new(self.path.as_path());
        let prefix = self.path.data.as_str();
        let mut local = f.sub_block();
        for function in self.functions.iter() {
            f.write_line(format_args!("function {}::{} ({{", prefix, function.name))?;
            let mut primitive_arguments = Vec::new();
            for (argument_name, argument) in function.arguments.iter() {
                let argument_guard = argument.read();
                let mut path_prefix = PathBuf::new(".");
                path_prefix.push(argument_name.as_str());
                argument_guard.data_type()
                    .expect("Function arguments cannot have undefined data type")
                    .make_primitives(path_prefix, &mut primitive_arguments);
            }
            for primitive in primitive_arguments {
                local.write_line(format_args!("{}: {},", primitive.path, TSQL(&primitive.field_type, parameters.clone())))?;
            }
            f.write_line("}) -> {")?;
            for primitive in function.result.primitives() {
                local.write_line(format_args!("{}: {},", primitive.path, TSQL(&primitive.field_type, parameters.clone())))?;
            }
            f.write_line("}")?;
            f.write_line("")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseProject {
    modules: HashMap<PathBuf, DatabaseModule>,
}

impl DatabaseProject {
    pub fn new(project: &IndexMap<SyncRef<PathBuf>, SyncRef<Module>>) -> Self {
        let mut modules = HashMap::new();
        for (module_path, module) in project.iter() {
            modules.insert(
                module_path.read().clone(),
                DatabaseModule::new(module),
            );
        }
        Self {
            modules,
        }
    }
    pub fn generate(&self, target: &mut impl Write) -> fmt::Result {
        let mut code_formatter = CodeFormatter::new(target);
        code_formatter.indent_size = 4;
        let mut root = code_formatter.root_block();
        let descriptions = root.sub_block();

        root.write_line("-- Tables")?;
        for (_, module) in self.modules.iter() {
            module.generate_tables(root.sub_block())?;
            root.write_line("")?;
        }
        root.write_line("")?;

        root.write_line("-- Functions")?;
        for (_, module) in self.modules.iter() {
            module.generate_functions(descriptions.clone())?;
        }
        root.write_line("")?;
        Ok(())
    }
    pub fn generate_string(&self) -> Result<String, fmt::Error> {
        let mut result = String::new();
        self.generate(&mut result)?;
        Ok(result)
    }
}
