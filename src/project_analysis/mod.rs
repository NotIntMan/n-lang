/*
    1. Resolver can load text, parse it and insert gotten module inside project.
        1.1. This load can be requested from outside.
        1.2. Begin of resolving process shall request load module with empty name.
        1.3. Every module can request to load another module.

    2. As said in #1.3, every module can request to load another module.
        2.1. Modules got context-object during resolving.
        2.2. Module is considered as unresolved until all it's dependencies are unresolved.

    3. As said in #2.2, resolution queue should contain dependencies of module before module itself.

    4. Resolution-context object should not be linked with project object (as it was before).
        4.1. There is not "stuck" status for resolution because of new model.

*/

pub mod project;

pub use self::project::*;

pub mod error;

pub use self::error::*;

pub mod resolve;

pub use self::resolve::*;

pub mod source;

pub use self::source::*;

pub mod module;

pub use self::module::*;

pub mod item;

pub use self::item::*;

pub mod module_context;

pub use self::module_context::*;
