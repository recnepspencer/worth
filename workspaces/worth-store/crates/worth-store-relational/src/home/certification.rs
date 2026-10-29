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
    /// Observe the physically admitted owner immediately before descriptor
    /// readmission/publication. The observer can arm existing fault and pause
    /// controls; it supplies neither descriptor bytes nor admission authority.
    #[doc(hidden)]
    pub fn certification_open_with_descriptor_observer(
        request: RelationalStoreHomeRequest,
        media: FilesystemMediaAdmission,
        observer: impl FnOnce(&ServingPhysicalRuntime),
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalStoreHomeOpenRefusal> {
        let (location, builder) = request.into_parts();
        let serving =
            super::open_physical_store(&location, PhysicalOpening::production().with_media(media))?;
        observer(&serving);
        Self::admit(serving, builder, execution)
    }

    /// Observe completed cold-port journal reads during production readmission.
    /// The observer receives only the requested position and existing counters;
    /// it cannot supply bytes or change admission. Its owner may park a child
    /// before the payload returns to Relational's selected-head readmission.
    #[doc(hidden)]
    pub fn certification_open_with_readmission_observer(
        request: RelationalStoreHomeRequest,
        observer: impl Fn(
                worth_relational::facade::publication::PatchStreamPosition,
                crate::RelationalStoreColdReadCounters,
            ) + Send
            + Sync
            + 'static,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalStoreHomeOpenRefusal> {
        let (location, builder) = request.into_parts();
        let serving = std::sync::Arc::new(super::open_physical_store(
            &location,
            PhysicalOpening::production(),
        )?);
        let mut source = crate::cold_source::StoreColdSource::new(serving.clone());
        source.observe_commit_reads(observer);
        Self::admit_with_source(serving, std::sync::Arc::new(source), builder, execution)
    }

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
            descriptor: _,
        } = self;
        drop(runtime);
        drop(cold_source);
        if let Ok(serving) = std::sync::Arc::try_unwrap(serving) {
            let _aborted = serving.abort();
        }
    }
}
