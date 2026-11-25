//! One non-cloneable Home wrapper owns the concrete lifecycle and Query service.
use std::ops::Deref;
use std::sync::Mutex;
use worth_query_execution::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime, product::WorthQueryProductBranch,
};

pub(super) type DirectoryHome =
    worth_store_application::WorldReadmittedApplicationHome<(), (), (), (), ()>;

/// Startup-only trait-object boundary keeping rustdoc auto-trait synthesis on
/// `&WorthQueryApplicationHome` (e.g. `WorthQueryHostSession`) from walking
/// `worth-store-application`'s internal `im::Vector` index types past recursion
/// limit 128, while preserving the exact `Send + Sync + Unpin` contract and
/// drop order of `DirectoryHome`.
trait DirectoryHomeBacking: Send + Sync {
    fn as_home(&self) -> &DirectoryHome;
    fn into_home(self: Box<Self>) -> DirectoryHome;
}

impl DirectoryHomeBacking for DirectoryHome {
    fn as_home(&self) -> &DirectoryHome {
        self
    }

    fn into_home(self: Box<Self>) -> DirectoryHome {
        *self
    }
}

pub(super) struct ErasedDirectoryHome(Box<dyn DirectoryHomeBacking>);

impl ErasedDirectoryHome {
    pub(super) fn new(home: DirectoryHome) -> Self {
        Self(Box::new(home))
    }

    pub(super) fn into_inner(self) -> DirectoryHome {
        self.0.into_home()
    }
}

impl Deref for ErasedDirectoryHome {
    type Target = DirectoryHome;

    fn deref(&self) -> &Self::Target {
        self.0.as_home()
    }
}

const _: fn() = || {
    fn assert_auto_traits<T: Send + Sync + Unpin>() {}
    assert_auto_traits::<ErasedDirectoryHome>();
};

/// Whether opening this Home initialized a brand-new Home or readmitted an
/// existing one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHomeOpenDisposition {
    Created,
    Opened,
}

pub struct WorthQueryApplicationHome<Schema, Program> {
    pub(super) query: WorthQueryProgramApplicationRuntime<Schema, Program>,
    pub(super) directory: Option<ErasedDirectoryHome>,
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
    pub fn home_identity(&self) -> Option<worth_foundational::facade::HomeIdentity> {
        self.directory.as_deref().map(DirectoryHome::home_identity)
    }
    pub fn home_incarnation(&self) -> Option<worth_foundational::facade::HomeIncarnation> {
        self.directory.as_deref().map(DirectoryHome::incarnation)
    }
}
