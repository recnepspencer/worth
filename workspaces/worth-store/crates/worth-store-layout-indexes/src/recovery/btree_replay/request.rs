use worth_store_budgets::PreExecutionBudgetEnvelope;
use worth_store_contracts::AcceptedHandoffReadiness;
use worth_store_physical_format::{
    InMemoryPhysicalFormatReplayArtifact, PhysicalPageId, PhysicalReference, PhysicalSegmentId,
    PhysicalStoreIdentity,
};
use worth_store_recovery_physics::PhysicalSourceSelection;
use worth_store_security::StoreCurrentSecurityScopeWitnessSet;

#[derive(Debug, Clone)]
pub struct BTreeReplayRequest<'a> {
    pub(super) catalog: &'a crate::BootstrapCatalogReadAdmission,
    pub(super) security: &'a StoreCurrentSecurityScopeWitnessSet,
    pub(super) segment: PhysicalSegmentId,
    pub(super) page: PhysicalPageId,
    pub(super) budget: PreExecutionBudgetEnvelope,
    pub(super) physical_source: BTreeReplayPhysicalSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BTreeReplayLocation {
    segment: PhysicalSegmentId,
    page: PhysicalPageId,
}

impl BTreeReplayLocation {
    pub const fn new(segment: PhysicalSegmentId, page: PhysicalPageId) -> Self {
        Self { segment, page }
    }
}

#[derive(Debug, Clone)]
pub struct BTreeReplayPhysicalSource {
    pub(super) readiness: AcceptedHandoffReadiness,
    pub(super) root_reference: PhysicalReference,
    pub(super) replay_artifact: InMemoryPhysicalFormatReplayArtifact,
    pub(super) expected_store_identity: PhysicalStoreIdentity,
    pub(super) durable_source: PhysicalSourceSelection,
}

impl BTreeReplayPhysicalSource {
    pub const fn new(
        readiness: AcceptedHandoffReadiness,
        root_reference: PhysicalReference,
        replay_artifact: InMemoryPhysicalFormatReplayArtifact,
        expected_store_identity: PhysicalStoreIdentity,
        durable_source: PhysicalSourceSelection,
    ) -> Self {
        Self {
            readiness,
            root_reference,
            replay_artifact,
            expected_store_identity,
            durable_source,
        }
    }
}

impl<'a> BTreeReplayRequest<'a> {
    pub fn new(
        catalog: &'a crate::BootstrapCatalogReadAdmission,
        security: &'a StoreCurrentSecurityScopeWitnessSet,
        location: BTreeReplayLocation,
        budget: PreExecutionBudgetEnvelope,
        physical_source: BTreeReplayPhysicalSource,
    ) -> Self {
        Self {
            catalog,
            security,
            segment: location.segment,
            page: location.page,
            budget,
            physical_source,
        }
    }
}
