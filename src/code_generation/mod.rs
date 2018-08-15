use helpers::{
    BlockFormatter,
    CodeFormatter,
    Generate,
    Map,
    Path,
    PathBuf,
    SimpleFormatter,
    SyncRef,
    TSQLParameters,
};
use indexmap::IndexMap;
use language::{
    DataType,
    FunctionDefinition,
    TableDefinition,
};
use project_analysis::Module;
use std::{
    fmt::{
        self,
        Write,
    },
};

#[derive(Debug, Clone)]
pub struct RPCModule {
    data_types: Map<String, DataType>,
    functions: Map<String, FunctionDefinition>,
    sub_modules: Map<String, RPCModule>,
}

impl RPCModule {
    pub fn top(project: &IndexMap<SyncRef<PathBuf>, SyncRef<Module>>) -> Self {
        let mut sub_modules = Map::new();
        for (path, module) in project {
            let path_guard = path.read();
            if let Some(name) = path_guard.the_only() {
                sub_modules.insert(name, RPCModule::new(module, project));
            }
        }
        sub_modules.sort();
        RPCModule {
            data_types: Map::new(),
            functions: Map::new(),
            sub_modules,
        }
    }
    pub fn new(source: &SyncRef<Module>, project: &IndexMap<SyncRef<PathBuf>, SyncRef<Module>>) -> Self {
        let source_guard = source.read();

        let mut data_types = Map::new();
        let mut functions = Map::new();
        let mut sub_modules = Map::new();

        for (item_name, item) in source_guard.items() {
            let item_guard = item.value.read();
            if !item_guard.is_belongs_to(source) { continue; }
            if let Some(data_type) = item_guard.get_data_type() {
                data_types.insert(item_name.as_str(), data_type.body.clone());
            } else if let Some(function) = item_guard.get_function() {
                functions.insert(item_name.as_str(), function.clone());
            } else if let Some(table) = item_guard.get_table() {
                sub_modules.insert(item_name.as_str(), RPCModule::for_table(table));
            }
        }

        let source_path_guard = source_guard.path().read();
        let source_path = source_path_guard.as_path();

        for (module_path, module) in project {
            let module_path_guard = module_path.read();
            if let Some(name) = source_path.is_begin_of(module_path_guard.as_path())
                .and_then(Path::the_only)
                {
                    sub_modules.insert(name, RPCModule::new(module, project));
                }
        }
        data_types.sort();
        functions.sort();
        sub_modules.sort();
        RPCModule {
            data_types,
            functions,
            sub_modules,
        }
    }
    pub fn for_table(table: &TableDefinition) -> Self {
        let mut data_types = Map::new();
        data_types.insert("entity", table.entity.clone());
        data_types.insert("primary_key", table.primary_key.clone());
        RPCModule {
            data_types,
            functions: Map::new(),
            sub_modules: Map::new(),
        }
    }
    pub fn fmt(&self, f: &mut SimpleFormatter, path: Path) -> fmt::Result {
        for (module_name, module) in self.sub_modules.iter() {
            writeln!(f, "export module {} {{", module_name)?;
            module.fmt(
                &mut f.sub_block(),
                PathBuf::from_paths(
                    path,
                    Path::new(module_name.as_str(), "::"),
                ).as_path(),
            )?;
            writeln!(f, "}}")?;
        }
        for (name, data_type) in self.data_types.iter() {
            data_type.fmt_export(f, &name)?;
        }
        for (_name, function) in self.functions.iter() {
            function.fmt_export(f, path)?;
        }
        Ok(())
    }
    pub fn generate_string(&self) -> Result<String, fmt::Error> {
        let mut result = String::new();
        {
            let mut formatter = SimpleFormatter::new(&mut result, 4);

            // Imports
            writeln!(formatter, "import * as _mssql from 'mssql'")?;
            writeln!(formatter, "")?;

            self.fmt(&mut formatter, Path::new("", "::"))?;
        }
        Ok(result)
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
        result.tables.sort_by(|a, b| a.name.cmp(&b.name));
        result.functions.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }
    pub fn generate_tables(&self, mut f: BlockFormatter<impl Write>) -> fmt::Result {
        let parameters = TSQLParameters::new(self.path.as_path());
        for table in self.tables.iter() {
            Generate::fmt(table, f.clone(), parameters.clone())?;
            f.write_line("GO")?;
            f.write_line("")?;
        }
        Ok(())
    }
    pub fn generate_functions(&self, mut f: BlockFormatter<impl Write>) -> fmt::Result {
        let parameters = TSQLParameters::new(self.path.as_path());
        for function in self.functions.iter() {
            Generate::fmt(function, f.clone(), parameters.clone())?;
            f.write_line("GO")?;
            f.write_line("")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseProject {
    modules: Map<PathBuf, DatabaseModule>,
}

impl DatabaseProject {
    pub fn new(project: &IndexMap<SyncRef<PathBuf>, SyncRef<Module>>) -> Self {
        let mut modules = Map::new();
        for (module_path, module) in project.iter() {
            modules.insert(
                module_path.read().clone(),
                DatabaseModule::new(module),
            );
        }
        modules.sort();
        Self {
            modules,
        }
    }
    pub fn generate(&self, target: &mut impl Write) -> fmt::Result {
        let mut code_formatter = CodeFormatter::new(target);
        code_formatter.indent_size = 4;
        let root = code_formatter.root_block();

        for (_, module) in self.modules.iter() {
            module.generate_tables(root.clone())?;
            module.generate_functions(root.clone())?;
        }

        Ok(())
    }
    pub fn generate_string(&self) -> Result<String, fmt::Error> {
        let mut result = String::new();
        self.generate(&mut result)?;
        Ok(result)
    }
}
