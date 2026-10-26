//! Certification-only doors: open a home on a Store the test opened itself
//! (with a media fault schedule), open a home through the production path
//! with a substituted media admission or a recovery that parks at a process
//! yieldpoint, reach the home's Store to park a publication at a mutation
//! seam, and abandon a home as a crash would. Each runs the production
//! admission path; none exists in a production build.

use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalRecoveryProcessYieldpoint, ServingPhysicalRuntime,
};

use super::{RelationalStoreHome, RelationalStoreHomeOpenRefusal, RelationalStoreHomeRequest};
use crate::physical_store::PhysicalOpening;

impl RelationalStoreHome {
    /// Admit the home on `serving` exactly as `open` would after its own
    /// physical open. The request's location is ignored.
    #[doc(hidden)]
    pub fn certification_open_on(
        serving: ServingPhysicalRuntime,
        request: RelationalStoreHomeRequest,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalStoreHomeOpenRefusal> {
        let (_, builder) = request.into_parts();
        Self::admit(serving, builder, execution)
    }

    /// `open`, with the Store served through `media` (for example a
    /// certification admission carrying a fault schedule). Initialization,
    /// production recovery, the home's Store profile and readmission are
    /// unchanged.
    #[doc(hidden)]
    pub fn certification_open_with_media(
        request: RelationalStoreHomeRequest,
        media: FilesystemMediaAdmission,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalStoreHomeOpenRefusal> {
        Self::open_through(
            request,
            PhysicalOpening::production().with_media(media),
            execution,
        )
    }

    /// `open`, with Store's production recovery armed at `yieldpoint`, so a
    /// parent process can kill recovery at that stage.
    #[doc(hidden)]
    pub fn certification_open_with_recovery_yieldpoint(
        request: RelationalStoreHomeRequest,
        yieldpoint: PhysicalRecoveryProcessYieldpoint,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalStoreHomeOpenRefusal> {
        Self::open_through(
            request,
            PhysicalOpening::production().with_recovery_yieldpoint(yieldpoint),
            execution,
        )
    }

    /// The home's Store, for certification seams.
    #[doc(hidden)]
    pub fn certification_serving(&self) -> &ServingPhysicalRuntime {
        &self.serving
    }

    /// Abandon the home as a crash would: release the runtime and abort the
    /// Store without its orderly close.
    #[doc(hidden)]
    pub fn certification_abort(self) {
        let Self {
            runtime,
            cold_source,
            serving,
        } = self;
        drop(runtime);
        drop(cold_source);
        if let Ok(serving) = std::sync::Arc::try_unwrap(serving) {
            let _aborted = serving.abort();
        }
    }
}
