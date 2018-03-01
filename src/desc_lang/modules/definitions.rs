use desc_lang::compounds::{
    DataType,
    Field,
};
use desc_lang::functions::FunctionDefinition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTypeDefinition<'source> {
    pub name: &'source str,
    pub body: DataType<'source>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableDefinition<'source> {
    pub name: &'source str,
    pub body: Vec<(&'source str, Field<'source>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalItemImport<'source> {
    pub path: Vec<&'source str>,
    pub alias: Option<&'source str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleDefinitionValue<'source> {
    DataType(DataTypeDefinition<'source>),
    Table(TableDefinition<'source>),
    Function(FunctionDefinition<'source>),
    Module(ModuleDefinition<'source>),
    Import(ExternalItemImport<'source>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDefinitionItem<'source> {
    pub public: bool,
    pub value: ModuleDefinitionValue<'source>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDefinition<'source> {
    name: &'source str,
    items: Vec<ModuleDefinitionItem<'source>>,
}

impl<'source> ModuleDefinitionValue<'source> {
    fn name(&self) -> &'source str {
        use self::ModuleDefinitionValue::*;
        match self {
            &DataType(ref def) => def.name,
            &Table(ref def) => def.name,
            &Function(ref def) => def.name,
            &Module(ref def) => def.name,
            &Import(ref def) => {
                def.alias.or_else(|| def.path.last().map(|name| *name));
                match def.alias {
                    Some(alias) => alias,
                    None => match def.path.last() {
                        Some(name) => *name,
                        None => panic!("Empty module path should not exists in definitions!"),
                    }
                }
            },
        }
    }
    fn into_named_pair(self) -> (&'source str, Self) {
        let name = self.name();
        (name, self)
    }
}

// TODO Подумать над идеей подмодулей и синтаксисом вызова функций из них
