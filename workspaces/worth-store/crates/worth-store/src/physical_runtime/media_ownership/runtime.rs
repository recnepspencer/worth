use worth_store_physical_backend::{
    FilesystemBackendProfile, QualifiedFilesystemMedia, QualifiedMediaCapabilities,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use crate::physical_runtime::{runtime::PhysicalRuntimeCore, RuntimeIdentity};
use crate::physical_runtime::{AbortedRuntime, ClosedRuntime};

use super::{MediaOwnedObservationPhase, MediaShutdownOutcome, PhysicalMediaObserver};

/// Sole move-only C.4 owner of the original runtime core and one qualified
/// real-filesystem owner.
pub struct MediaOwnedPhysicalRuntime {
    termination: crate::physical_runtime::lifecycle::LifecycleTerminationGuard,
    media: QualifiedFilesystemMedia,
    core: PhysicalRuntimeCore,
    root_protocol_counters: crate::physical_runtime::RootProtocolRouteCounterCells,
}

impl MediaOwnedPhysicalRuntime {
    pub(in crate::physical_runtime) const fn record_serving_media(
        &self,
    ) -> &QualifiedFilesystemMedia {
        &self.media
    }

    pub(in crate::physical_runtime) fn into_record_serving_parts(
        self,
    ) -> (
        crate::physical_runtime::lifecycle::LifecycleTerminationGuard,
        QualifiedFilesystemMedia,
        PhysicalRuntimeCore,
    ) {
        (self.termination, self.media, self.core)
    }

    pub fn initialize_record_store(
        self,
        request: crate::physical_runtime::PhysicalRecordInitialization,
    ) -> crate::physical_runtime::RecordStoreInitializationOutcome {
        crate::physical_runtime::record_serving::initialize(self, request)
    }

    pub fn open_record_store(
        self,
        request: crate::physical_runtime::PhysicalRecordOpen,
    ) -> crate::physical_runtime::RecordStoreOpenOutcome {
        crate::physical_runtime::record_serving::open(self, request)
    }

    pub(super) fn new(core: PhysicalRuntimeCore, media: QualifiedFilesystemMedia) -> Self {
        let termination = core.termination_guard();
        Self {
            termination,
            media,
            core,
            root_protocol_counters: crate::physical_runtime::RootProtocolRouteCounterCells::default(
            ),
        }
    }

    pub const fn runtime_identity(&self) -> RuntimeIdentity {
        self.core.runtime_identity()
    }

    pub(in crate::physical_runtime) fn lifecycle_generation(
        &self,
    ) -> crate::physical_runtime::LifecycleGeneration {
        self.core.lifecycle_generation()
    }

    pub(in crate::physical_runtime) fn lifecycle_state(
        &self,
    ) -> std::sync::Arc<crate::physical_runtime::lifecycle::LifecycleState> {
        self.core.lifecycle_state()
    }

    pub(in crate::physical_runtime) const fn root_protocol_counter_cells(
        &self,
    ) -> &crate::physical_runtime::RootProtocolRouteCounterCells {
        &self.root_protocol_counters
    }

    pub fn root_protocol_counters(&self) -> crate::physical_runtime::RootProtocolRouteCounters {
        self.root_protocol_counters.snapshot()
    }

    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.media.store_identity()
    }

    pub fn backend_profile(&self) -> &FilesystemBackendProfile {
        self.media.profile()
    }

    pub fn capabilities(&self) -> &QualifiedMediaCapabilities {
        self.media.capabilities()
    }

    pub fn physical_durability_admission_basis(
        &self,
    ) -> Result<
        worth_store_physical_backend::PhysicalDurabilityAdmissionBasis,
        worth_store_physical_backend::BackendCapabilityAdmissionDenial,
    > {
        self.media.physical_durability_admission_basis()
    }

    pub fn qualification_report(
        &self,
    ) -> worth_store_physical_backend::RootProfileQualificationReport {
        self.media.qualification_report()
    }

    pub fn media_counters(&self) -> worth_store_physical_backend::MediaCounterSnapshot {
        self.media.counters()
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_operation_summary(
        &self,
    ) -> Result<
        crate::physical_runtime::certification::MediaOperationSummary,
        crate::physical_runtime::certification::MediaEvidenceLoweringDenial,
    > {
        crate::physical_runtime::media_evidence::MediaOperationSummary::from_qualified_media(
            &self.media,
        )
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_confinement_probe(
        &self,
        authority: crate::physical_runtime::certification::CertificationMediaFaultAuthority,
        component: &str,
    ) -> Result<(), worth_store_physical_backend::NamespaceConfinementDenial> {
        self.media
            .certification_confinement_probe(authority, component)
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_staging_effect_probe(
        &self,
        authority: crate::physical_runtime::certification::CertificationMediaFaultAuthority,
        component: &str,
    ) -> worth_store_physical_backend::CertificationConfinementEffect {
        self.media
            .certification_staging_effect_probe(authority, component)
    }

    pub fn observer(&self) -> PhysicalMediaObserver<MediaOwnedObservationPhase> {
        let (lifecycle, lease) = self.core.media_observation_parts();
        PhysicalMediaObserver::for_media_owned(
            self.runtime_identity(),
            self.store_identity(),
            self.media.mutation_owner(),
            self.media.profile().clone(),
            self.media.counter_observer(),
            lifecycle,
            lease,
        )
    }

    pub fn close(self) -> MediaShutdownOutcome<ClosedRuntime> {
        let Self {
            termination,
            media,
            core,
            root_protocol_counters: _,
        } = self;
        drop(termination);
        let release = media.close();
        MediaShutdownOutcome::new(core.close(), release)
    }

    pub fn abort(self) -> MediaShutdownOutcome<AbortedRuntime> {
        let Self {
            termination,
            media,
            core,
            root_protocol_counters: _,
        } = self;
        drop(termination);
        let release = media.close();
        MediaShutdownOutcome::new(core.abort(), release)
    }
}
