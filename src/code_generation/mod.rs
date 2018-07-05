use indexmap::IndexMap;
use helpers::{
    Map,
    PathBuf,
    SyncRef,
};
use language::{
    CompoundDataType,
    DataType,
    FunctionDefinition,
    TableDefinition,
};
use project_analysis::Module;

#[derive(Debug, Clone)]
pub struct DataClass {
    path: PathBuf,
    fields: Map<String, DataType>,
}

impl DataClass {
    pub fn new(path: PathBuf, reflection_target: &DataType, mut request_data_class: impl FnMut(PathBuf, &DataType) -> ()) -> Option<Self> {
        match reflection_target {
            DataType::Compound(CompoundDataType::Structure(fields)) => {
                let mut result_fields = Map::new();
                for (field_name, field) in fields.iter() {
                    let mut sub_path = path.clone();
                    sub_path.push(field_name.as_str());
                    request_data_class(sub_path, &field.field_type);
                    result_fields.insert(field_name.as_str(), field.field_type.clone());
                }
                Some(Self {
                    path,
                    fields: result_fields,
                })
            }
            DataType::Compound(CompoundDataType::Tuple(fields)) => {
                let mut result_fields = Map::new();
                for (index, field) in fields.iter().enumerate() {
                    let field_name = format!("component{}", index);
                    let mut sub_path = path.clone();
                    sub_path.push(field_name.as_str());
                    request_data_class(sub_path, &field.field_type);
                    result_fields.insert(field_name, field.field_type.clone());
                }
                Some(Self {
                    path,
                    fields: result_fields,
                })
            }
            DataType::Reference(item) => {
                let item = item.read();
                let data_type = item.get_data_type()?;
                request_data_class(path, &data_type.body);
                None
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RPCModule {
    path: PathBuf,
    functions: Vec<FunctionDefinition>,
}

impl RPCModule {
    pub fn new(source: &Module, mut request_data_class: impl FnMut(PathBuf, &DataType) -> ()) -> Self {
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
                for (argument_name, argument_type) in function.arguments.iter() {
                    let mut sub_path = function_path.clone();
                    sub_path.push(argument_name.as_str());
                    request_data_class(sub_path, argument_type);
                }
                function_path.push("result");
                request_data_class(function_path, &function.result);
                result.functions.push(function);
            }
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct RPCProject {
    rpc_modules: Map<String, RPCModule>,
    data_classes: Map<PathBuf, DataClass>,
    anonymous_data_classes: Map<PathBuf, DataClass>,
}

impl RPCProject {
    pub fn new(project: &IndexMap<SyncRef<PathBuf>, SyncRef<Module>>) -> Self {
        let mut data_classes: Map<PathBuf, DataClass> = Map::new();
        let mut requested_data_classes: Map<PathBuf, DataType> = Map::new();
        for (module_path, module) in project.iter() {
            let module_path_guard = module_path.read();
            let module_guard = module.read();
            for (item_name, item) in module_guard.items().iter() {
                let item_value = item.value.read();
                if let Some(data_type_def) = item_value.get_data_type() {
                    let mut path = module_path_guard.clone();
                    path.push(item_name.as_str());
                    if let Some(data_class) = DataClass::new(
                        path.clone(),
                        &data_type_def.body,
                        |path, data_type| {
                            if !requested_data_classes.has(&path) {
                                requested_data_classes.insert(path, data_type.clone());
                            }
                        },
                    ) {
                        data_classes.insert(path, data_class);
                    }
                }
            }
            let rpc_module = RPCModule::new(
                &*module_guard,
                |path, data_type| {},
            );
        }
        unimplemented!()
    }
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
