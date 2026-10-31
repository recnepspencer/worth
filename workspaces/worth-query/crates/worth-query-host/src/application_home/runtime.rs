//! One non-cloneable Home wrapper owns the concrete lifecycle and Query service.
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;

pub(super) type DirectoryHome =
    worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>;

pub struct WorthQueryApplicationHome<Schema, Program> {
    pub(super) query: WorthQueryProgramApplicationRuntime<Schema, Program>,
    pub(super) directory: Option<DirectoryHome>,
}
impl<Schema, Program> WorthQueryApplicationHome<Schema, Program> {
    pub fn runtime(&self) -> &WorthQueryProgramApplicationRuntime<Schema, Program> {
        &self.query
    }
    pub fn is_directory(&self) -> bool {
        self.directory.is_some()
    }
}
