//! One non-cloneable Home wrapper owns the concrete lifecycle and Query service.
use std::sync::Mutex;
use worth_query_execution::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime, product::WorthQueryProductBranch,
};

pub(super) type DirectoryHome =
    worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>;

/// Whether opening this Home initialized a brand-new Home or readmitted an
/// existing one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHomeOpenDisposition {
    Created,
    Opened,
}

pub struct WorthQueryApplicationHome<Schema, Program> {
    pub(super) query: WorthQueryProgramApplicationRuntime<Schema, Program>,
    pub(super) directory: Option<DirectoryHome>,
    pub(super) checkpoint_limits: worth_store_application::ApplicationHomeCheckpointLimits,
    pub(super) open_disposition: WorthQueryHomeOpenDisposition,
    pub(super) active_head: Mutex<WorthQueryProductBranch>,
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
    /// The installed primary graph belonging to this Home.
    pub fn primary_graph(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.query.runtime()
    }
    pub fn is_directory(&self) -> bool {
        self.directory.is_some()
    }
    /// Reports whether this opening created a new Home or reopened an existing one.
    pub const fn open_disposition(&self) -> WorthQueryHomeOpenDisposition {
        self.open_disposition
    }
    /// The branch currently targeted by `home.session(...)` and `home.fork_preview(...)`.
    pub fn active_branch(&self) -> WorthQueryProductBranch {
        *self
            .active_head
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
