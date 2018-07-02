use std::collections::HashMap;
use helpers::PathBuf;
use language::{
    DataType,
    FunctionDefinition,
    TableDefinition,
};

#[derive(Debug, Clone, PartialEq)]
pub struct DataClass {
    name: String,
    field: HashMap<String, DataType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RPCModule {
    name: String,
    functions: Vec<FunctionDefinition>,
}

#[derive(Debug, Clone)]
pub struct RPCProject {
    data_classes: HashMap<PathBuf, DataClass>,
    rpc_modules: HashMap<String, RPCModule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseModule {
    tables: Vec<TableDefinition>,
    functions: Vec<FunctionDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseProject {
    modules: HashMap<String, TableDefinition>,
}
