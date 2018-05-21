use helpers::SyncRef;
use language::DataSource;
use project_analysis::FunctionVariableScope;

#[derive(Debug, Clone, PartialEq)]
pub struct InsertSourceContext<'a> {
    pub scope: &'a SyncRef<FunctionVariableScope>,
    pub target: &'a DataSource,
}
