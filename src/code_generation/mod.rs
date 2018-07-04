//use helpers::PathBuf;
use helpers::Map;
use language::{
    DataType,
    FunctionDefinition,
    TableDefinition,
};

#[derive(Debug, Clone)]
pub struct DataClass {
    name: String,
    field: Map<String, DataType>,
}

#[derive(Debug, Clone)]
pub struct RPCModule {
    name: String,
    functions: Vec<FunctionDefinition>,
}

#[derive(Debug, Clone)]
pub struct RPCProject {
    data_classes: Map<DataType, DataClass>,
    rpc_modules: Map<String, RPCModule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DatabaseModule {
    tables: Vec<TableDefinition>,
    functions: Vec<FunctionDefinition>,
}

#[derive(Debug, Clone)]
pub struct DatabaseProject {
    modules: Map<String, TableDefinition>,
}
