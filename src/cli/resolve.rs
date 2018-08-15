use n_lang::{
    code_generation::{
        DatabaseProject,
        RPCModule,
    },
    helpers::{
        SyncRef,
        Resolve,
    },
    project_analysis::{
        SemanticErrors,
        HashMapSource,
        ProjectContext,
    },
};
use std::{
    error::Error,
    path::Path,
};
use stdlib::build_ms_sql_std_lib;

pub fn resolve_dir(path: &Path) -> Result<(DatabaseProject, RPCModule), Box<Error>> {
    let sources = HashMapSource::for_dir(path)?;
    let project_context = ProjectContext::new(SyncRef::new(build_ms_sql_std_lib()));
    for (module_path, _) in sources.texts() {
        project_context.request_resolving_module(module_path.as_path());
    }
    let project = project_context.resolve(&sources)
        .map_err(|errors| SemanticErrors::from(errors))?;
    Ok((
        DatabaseProject::new(&project),
        RPCModule::top(&project),
    ))
}
