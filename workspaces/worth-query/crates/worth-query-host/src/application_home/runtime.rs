//! One non-cloneable Home wrapper owns the concrete lifecycle and Query service.
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;

pub(super) type DirectoryHome =
    worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>;

pub struct WorthQueryApplicationHome<Schema, Program> {
    pub(super) query: WorthQueryProgramApplicationRuntime<Schema, Program>,
    pub(super) directory: Option<DirectoryHome>,
    pub(super) checkpoint_limits: worth_store_application::ApplicationHomeCheckpointLimits,
}
impl<Schema, Program> WorthQueryApplicationHome<Schema, Program>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchemaComposition,
{
    /// Retain this installation's invariant projection for application command owners.
    /// Reopening retains the readmitted installation; no initializer is replayed.
    pub fn retain_invariant_projection_authority(
        &self,
    ) -> worth_query_execution::facade::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<Schema>{
        worth_query_execution::facade::application_home_integration::retain_home_invariant_projection(
            &self.query,
        )
    }
}
impl<Schema, Program> WorthQueryApplicationHome<Schema, Program> {
    pub fn runtime(&self) -> &WorthQueryProgramApplicationRuntime<Schema, Program> {
        &self.query
    }
    pub fn is_directory(&self) -> bool {
        self.directory.is_some()
    }
    pub fn home_identity(&self) -> Option<worth_foundational::facade::HomeIdentity> {
        self.directory.as_ref().map(DirectoryHome::home_identity)
    }
    pub fn home_incarnation(&self) -> Option<worth_foundational::facade::HomeIncarnation> {
        self.directory.as_ref().map(DirectoryHome::incarnation)
    }
}
