use std::ops::Range;

use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest};

use super::super::{
    access::{extent_read_session::ExtentReadState, record_chunk_view::RecordReadIdentity},
    residency::{frame_loading::LoadedPhysicalFrame, PhysicalResidencyWorkPort},
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, ExternalPhysicalRecordLocator,
    PhysicalLocatorReadmissionOutcome, PhysicalRecordId, RecordReadDenial, RecordReadError,
    RecordReadLimits, RecordReadObservation,
};

mod cancellation;
mod extent;
#[path = "locate/failure_classification.rs"]
pub(super) mod failure_classification;
mod inline;
mod session;
pub use cancellation::RecordReadCancellation;
#[cfg(test)]
pub(in crate::physical_runtime) use failure_classification::assert_actual_lifecycle_manifest_denial_maps_without_damage;
use failure_classification::manifest_failure;

#[allow(
    clippy::large_enum_variant,
    reason = "inline frame authority stays move-owned to avoid a heap allocation on every ordinary inline record read"
)]
enum ReadPlacement {
    Inline {
        frame: LoadedPhysicalFrame,
        payload: Range<usize>,
        offset: usize,
    },
    Extent(Box<ExtentReadState>),
}

/// A live, bounded read of one physical record.
///
/// The session owns the read allocation and at most one current frame. Use
/// `next_chunk` to borrow decoded payload bytes or `read_next` to copy into a
/// caller-provided buffer. A borrowed chunk cannot outlive or advance this
/// session.
pub struct RecordReadSession {
    placement: ReadPlacement,
    identity: RecordReadIdentity,
    observation: RecordReadObservation,
    runtime: std::sync::Weak<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
    health_permit: super::super::lifecycle::serving_health::ServingHealthPermit,
    _lifecycle: super::super::lifecycle::record_lifecycle::RecordReadSessionLease,
    _allocation: worth_store_buffer_pool::OperationAllocationGrant,
}

/// Opens bounded record-read sessions through the serving Store.
///
/// The reader exposes no pool, frame, pin, eviction, or source-loading
/// authority.
pub struct PhysicalRecordReader {
    pub(in crate::physical_runtime::record_serving) store: StableStoreIdentity,
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) current_root: DurablePhysicalRootManifest,
    pub(in crate::physical_runtime::record_serving) generation:
        crate::physical_runtime::LifecycleGeneration,
    pub(in crate::physical_runtime::record_serving) runtime:
        std::sync::Weak<crate::physical_runtime::instance::PhysicalStoreWorkRuntime>,
    pub(in crate::physical_runtime::record_serving) lifecycle:
        super::super::lifecycle::record_lifecycle::RecordReaderLease,
    pub(in crate::physical_runtime::record_serving) residency: PhysicalResidencyWorkPort,
}

impl PhysicalRecordReader {
    /// Locates a record and opens a bounded read session.
    ///
    /// The caller-supplied limits are checked before payload streaming. A
    /// pressure denial is available through `RecordReadError::pressure`.
    pub fn open(
        &self,
        record: PhysicalRecordId,
        limits: RecordReadLimits,
    ) -> Result<RecordReadSession, RecordReadError> {
        let mut observation = RecordReadObservation::default();
        let runtime = self.runtime.upgrade().ok_or_else(|| {
            RecordReadError::new(RecordReadDenial::ServingRequiresInspection, observation)
        })?;
        runtime.health.permit().map_err(|_| {
            RecordReadError::new(RecordReadDenial::ServingRequiresInspection, observation)
        })?;
        let allocation = self.begin_record_read_allocation(record, observation)?;
        let mut discovery =
            super::super::access::manifest_routing::ManifestDiscoveryCounterSnapshot::default();
        let placement = super::super::access::manifest_routing::ManifestReader::serving(
            self.residency.clone(),
            self.format,
            self.access,
            self.current_root.clone(),
        )
        .locate(&allocation, record.persisted(), &mut discovery);
        observation.observe_manifest(discovery);
        let placement = placement.map_err(|failure| {
            self.read_error_for_record(record, manifest_failure(failure), observation)
        })?;
        let placement = placement
            .ok_or_else(|| RecordReadError::new(RecordReadDenial::RecordNotFound, observation))?;
        self.require_caller_limit(placement.payload_bytes(), limits, &mut observation)?;
        self.open_known_placement_with_allocation(record, placement, observation, allocation)
    }

    pub(in crate::physical_runtime::record_serving) fn open_known_placement(
        &self,
        record: PhysicalRecordId,
        placement: CurrentPhysicalRecordPlacement,
        limits: RecordReadLimits,
        mut observation: RecordReadObservation,
    ) -> Result<RecordReadSession, RecordReadError> {
        self.require_caller_limit(placement.payload_bytes(), limits, &mut observation)?;
        let allocation = self.begin_record_read_allocation(record, observation)?;
        self.open_known_placement_with_allocation(record, placement, observation, allocation)
    }

    pub(in crate::physical_runtime::record_serving) fn begin_read_allocation(
        &self,
        observation: RecordReadObservation,
    ) -> Result<worth_store_buffer_pool::OperationAllocationGrant, RecordReadError> {
        self.begin_read_allocation_with_basis(
            observation,
            super::super::PhysicalRecordPressureBasis::for_store(self.store),
        )
    }

    fn begin_record_read_allocation(
        &self,
        record: PhysicalRecordId,
        observation: RecordReadObservation,
    ) -> Result<worth_store_buffer_pool::OperationAllocationGrant, RecordReadError> {
        self.begin_read_allocation_with_basis(
            observation,
            super::super::PhysicalRecordPressureBasis::for_store(self.store).with_record(record),
        )
    }

    fn begin_read_allocation_with_basis(
        &self,
        observation: RecordReadObservation,
        basis: super::super::PhysicalRecordPressureBasis,
    ) -> Result<worth_store_buffer_pool::OperationAllocationGrant, RecordReadError> {
        self.residency
            .begin_operation(
                worth_store_buffer_pool::PhysicalOperationAllocationScope::ForegroundRead,
                std::num::NonZeroU64::new(u64::from(self.format.declaration().page_size().bytes()))
                    .expect("an admitted format page size is nonzero"),
            )
            .map_err(|reason| {
                self.read_error_with_basis(
                    RecordReadDenial::from_residency(reason),
                    observation,
                    basis,
                )
            })
    }

    fn require_caller_limit(
        &self,
        payload_bytes: u64,
        limits: RecordReadLimits,
        observation: &mut RecordReadObservation,
    ) -> Result<(), RecordReadError> {
        observation.requested_bytes = payload_bytes;
        if payload_bytes > u64::from(limits.maximum_payload.get()) {
            return Err(RecordReadError::new(
                RecordReadDenial::CallerLimitExceeded,
                *observation,
            ));
        }
        Ok(())
    }

    fn open_known_placement_with_allocation(
        &self,
        record: PhysicalRecordId,
        placement: CurrentPhysicalRecordPlacement,
        mut observation: RecordReadObservation,
        allocation: worth_store_buffer_pool::OperationAllocationGrant,
    ) -> Result<RecordReadSession, RecordReadError> {
        let runtime = self.runtime.upgrade().ok_or_else(|| {
            RecordReadError::new(RecordReadDenial::ServingRequiresInspection, observation)
        })?;
        runtime.health.permit().map_err(|_| {
            RecordReadError::new(RecordReadDenial::ServingRequiresInspection, observation)
        })?;
        let result = match placement {
            CurrentPhysicalRecordPlacement::Inline(value) => {
                self.open_inline(record, value, &mut observation, allocation)
            }
            CurrentPhysicalRecordPlacement::Extent(value) => {
                self.open_extent(record, value, &mut observation, allocation)
            }
        };
        result.map_err(|denial| self.read_error_for_record(record, denial, observation))
    }

    /// Revalidates an external locator against the current Store generation.
    pub fn readmit_locator(
        &self,
        locator: ExternalPhysicalRecordLocator,
    ) -> PhysicalLocatorReadmissionOutcome {
        super::super::access::readmission::readmit_locator(self, locator)
    }

    /// Revalidates an external locator and opens its bounded read session.
    pub fn open_external(
        &self,
        locator: ExternalPhysicalRecordLocator,
        limits: RecordReadLimits,
    ) -> Result<RecordReadSession, RecordReadError> {
        let mut observation = RecordReadObservation::default();
        let allocation = self.begin_read_allocation(observation)?;
        let readmitted =
            super::super::access::readmission::readmit_locator_detailed(self, &allocation, locator)
                .map_err(|failure| self.read_error(failure.read_denial(), failure.observation()))?;
        observation = readmitted.observation();
        self.require_caller_limit(
            readmitted.placement().payload_bytes(),
            limits,
            &mut observation,
        )?;
        self.open_known_placement_with_allocation(
            readmitted.record(),
            readmitted.placement(),
            observation,
            allocation,
        )
    }

    /// Returns the stable physical Store identity read by this facade.
    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }

    fn read_error(
        &self,
        denial: RecordReadDenial,
        observation: RecordReadObservation,
    ) -> RecordReadError {
        self.read_error_with_basis(
            denial,
            observation,
            super::super::PhysicalRecordPressureBasis::for_store(self.store),
        )
    }

    fn read_error_for_record(
        &self,
        record: PhysicalRecordId,
        denial: RecordReadDenial,
        observation: RecordReadObservation,
    ) -> RecordReadError {
        self.read_error_with_basis(
            denial,
            observation,
            super::super::PhysicalRecordPressureBasis::for_store(self.store).with_record(record),
        )
    }

    fn read_error_with_basis(
        &self,
        denial: RecordReadDenial,
        observation: RecordReadObservation,
        basis: super::super::PhysicalRecordPressureBasis,
    ) -> RecordReadError {
        if let Some(runtime) = self.runtime.upgrade() {
            runtime.health.observe_read_denial(denial);
        }
        if let RecordReadDenial::ResidencyUnavailable(reason) = denial {
            if let Some(pressure) = super::super::PhysicalRecordPressureEvidence::from_failure(
                reason,
                self.generation,
                basis,
            ) {
                return RecordReadError::from_pressure(pressure, observation);
            }
        }
        RecordReadError::new(denial, observation)
    }
}
