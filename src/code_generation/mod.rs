//use helpers::PathBuf;
use helpers::Map;
use language::{
    DataType,
    FunctionDefinition,
    TableDefinition,
};
use project_analysis::module::Module;
use helpers::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DataClass {
    name: String,
    fields: Map<String, DataType>,
}

impl DataClass {
    pub fn new(name: &str, reflection_target: &DataType) -> Option<Self> {
        let data_type = match reflection_target {
            DataType::Compound(data_type) => data_type,
            DataType::Reference(item) => {
                let item = item.read();
                let data_type = item.get_data_type()?;
                return DataClass::new(
                    name,
                    &data_type.body,
                );
            }
            _ => return None,
        };
        let mut result = Self {
            name: name.to_string(),
            fields: Map::new(),
        };
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub struct RPCModule {
    path: PathBuf,
    functions: Vec<FunctionDefinition>,
}

impl RPCModule {
    pub fn new(source: &Module) -> Self {
        let mut result = Self {
            path: source.path().read().clone(),
            functions: Vec::new(),
        };
        for (item_name, item) in source.items().iter() {
            let item = item.value.read();
            // TODO Создание DataClass'ов для таблиц аргументов и возвращаемых значений функций
            if let Some(function) = item.get_function() {
                let mut function = function.clone();
                function.name = item_name.clone();
                result.functions.push(function);
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct RPCProject {
    data_classes: Map<DataType, DataClass>,
    rpc_modules: Map<String, RPCModule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseModule {
    path: PathBuf,
    tables: Vec<TableDefinition>,
    functions: Vec<FunctionDefinition>,
}

impl DatabaseModule {
    pub fn new(source: &Module) -> Self {
        let mut result = Self {
            path: source.path().read().clone(),
            tables: Vec::new(),
            functions: Vec::new(),
        };
        for (item_name, item) in source.items().iter() {
            let item = item.value.read();
            if let Some(table) = item.get_table() {
                let mut table = table.clone();
                table.name = item_name.clone();
                result.tables.push(table);
            }
            if let Some(function) = item.get_function() {
                let mut function = function.clone();
                function.name = item_name.clone();
                result.functions.push(function);
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseProject {
    modules: Map<String, TableDefinition>,
}
