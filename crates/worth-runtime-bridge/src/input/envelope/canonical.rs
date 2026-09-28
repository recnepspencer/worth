use super::*;
use crate::error::BridgeRouteError;
use crate::mapping::TruthDeltaSurfaceKind;
use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCommittedPatchItem {
    entity_identity: Arc<str>,
    relational_record_identity: Option<RelationalBridgeRecordIdentityParts>,
    target: BridgeCommittedPatchTarget,
    semantic_change: Option<BridgeSemanticAspectChange>,
}

impl BridgeCommittedPatchItem {
    pub fn with_target(
        entity_identity: impl Into<Arc<str>>,
        target: BridgeCommittedPatchTarget,
    ) -> Self {
        Self {
            entity_identity: entity_identity.into(),
            relational_record_identity: None,
            target,
            semantic_change: None,
        }
    }

    pub fn entity_identity(&self) -> &str {
        self.entity_identity.as_ref()
    }

    pub fn relational_record_identity_parts(&self) -> Option<RelationalBridgeRecordIdentityParts> {
        self.relational_record_identity
    }

    pub(crate) fn from_relational_record_parts(
        entity_identity: impl Into<Arc<str>>,
        relational_record_identity: RelationalBridgeRecordIdentityParts,
        target: BridgeCommittedPatchTarget,
        semantic_change: Option<BridgeSemanticAspectChange>,
    ) -> Self {
        Self {
            entity_identity: entity_identity.into(),
            relational_record_identity: Some(relational_record_identity),
            target,
            semantic_change,
        }
    }

    pub fn target(&self) -> &BridgeCommittedPatchTarget {
        &self.target
    }

    pub fn aspect_key(&self) -> &AspectKey {
        self.target.aspect_key()
    }

    pub fn aspect_locator(&self) -> &AspectLocator {
        self.target.aspect_locator()
    }

    pub fn field_locator(&self) -> Option<&AspectFieldLocator> {
        self.target.field_locator()
    }

    pub fn mutation_mask(&self) -> &AspectMask<MutationMask> {
        self.target.mutation_mask()
    }

    pub fn projection_mask(&self) -> &AspectMask<ProjectionMask> {
        self.target.projection_mask()
    }

    pub fn surface_kind(&self) -> TruthDeltaSurfaceKind {
        self.target.surface_kind()
    }

    pub fn semantic_change(&self) -> Option<&BridgeSemanticAspectChange> {
        self.semantic_change.as_ref()
    }

    pub(crate) fn target_canonical_basis(&self) -> String {
        self.target.canonical_basis()
    }

    pub(crate) fn canonical_basis(&self) -> String {
        let semantic = self.semantic_change.as_ref().map_or_else(
            || "none".to_string(),
            BridgeSemanticAspectChange::canonical_basis,
        );
        format!(
            "{}:{}{}:{}",
            self.target_canonical_basis().len(),
            self.target_canonical_basis(),
            semantic.len(),
            semantic,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCommittedPatchBody {
    canonical_items: Vec<BridgeCommittedPatchItem>,
    canonical_record_changes: Vec<BridgeCommittedRecordChange>,
}

impl BridgeCommittedPatchBody {
    pub fn new(
        canonical_items: Vec<BridgeCommittedPatchItem>,
        canonical_record_changes: Vec<BridgeCommittedRecordChange>,
    ) -> Self {
        Self {
            canonical_items,
            canonical_record_changes,
        }
    }

    pub fn canonical_record_changes(&self) -> &[BridgeCommittedRecordChange] {
        &self.canonical_record_changes
    }

    pub fn canonical_items(&self) -> &[BridgeCommittedPatchItem] {
        &self.canonical_items
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCommittedPatchSummary {
    patch_item_count: usize,
    normalized_patch_item_count: usize,
    record_change_count: usize,
    authoritative_lowering: BridgeAuthoritativePatchLoweringCounters,
}

impl BridgeCommittedPatchSummary {
    pub fn new(
        patch_item_count: usize,
        normalized_patch_item_count: usize,
        record_change_count: usize,
        authoritative_lowering: BridgeAuthoritativePatchLoweringCounters,
    ) -> Self {
        Self {
            patch_item_count,
            normalized_patch_item_count,
            record_change_count,
            authoritative_lowering,
        }
    }

    pub fn record_change_count(&self) -> usize {
        self.record_change_count
    }

    pub fn patch_item_count(&self) -> usize {
        self.patch_item_count
    }

    pub fn normalized_patch_item_count(&self) -> usize {
        self.normalized_patch_item_count
    }

    pub fn authoritative_lowering(&self) -> &BridgeAuthoritativePatchLoweringCounters {
        &self.authoritative_lowering
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BridgeAuthoritativePatchLoweringCounters {
    pub source_record_patches_examined: u64,
    pub source_record_patches_filtered_out: u64,
    pub record_patches_inspected: u64,
    pub authoritative_operations_inspected: u64,
    pub expected_operations_materialized: u64,
    pub semantic_changes_inspected: u64,
    pub semantic_changes_matched: u64,
    pub semantic_changes_emission_classified: u64,
    pub field_targets_emitted: u64,
    pub whole_aspect_targets_emitted: u64,
    pub endpoint_targets_emitted: u64,
    pub lifecycle_targets_emitted: u64,
    pub opaque_changes_emitted: u64,
    pub declared_widenings: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeCommittedPatchEnvelope(BridgeEnvelopeCore<CanonicalPatchEnvelopeBody>);

impl BridgeCommittedPatchEnvelope {
    pub(super) fn from_core(core: BridgeEnvelopeCore<CanonicalPatchEnvelopeBody>) -> Self {
        Self(core)
    }

    pub fn new(
        envelope_identity: BridgeCommittedPatchEnvelopeIdentity,
        patch_items: Vec<BridgeCommittedPatchItem>,
    ) -> Result<Self, BridgeRouteError> {
        construction::construct_committed_patch_envelope(envelope_identity, patch_items)
    }

    pub fn new_with_record_changes(
        envelope_identity: BridgeCommittedPatchEnvelopeIdentity,
        patch_items: Vec<BridgeCommittedPatchItem>,
        record_changes: Vec<BridgeCommittedRecordChange>,
    ) -> Result<Self, BridgeRouteError> {
        construction::construct_committed_patch_envelope_with_record_changes(
            envelope_identity,
            patch_items,
            record_changes,
            BridgeAuthoritativePatchLoweringCounters::default(),
        )
    }

    pub fn new_with_authoritative_lowering(
        envelope_identity: BridgeCommittedPatchEnvelopeIdentity,
        patch_items: Vec<BridgeCommittedPatchItem>,
        record_changes: Vec<BridgeCommittedRecordChange>,
        counters: BridgeAuthoritativePatchLoweringCounters,
    ) -> Result<Self, BridgeRouteError> {
        construction::construct_committed_patch_envelope_with_record_changes(
            envelope_identity,
            patch_items,
            record_changes,
            counters,
        )
    }

    pub fn producer_metadata(&self) -> &BridgeProducerMetadata {
        self.0.producer_metadata()
    }

    pub fn commit_identity(&self) -> &TruthCommitIdentity {
        self.0.commit_identity()
    }

    pub fn patch_identity(&self) -> &TruthPatchIdentity {
        self.0.patch_identity()
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        self.0.snapshot_identity()
    }

    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        self.0.branch_identity()
    }

    pub fn patch_summary(&self) -> &BridgeCommittedPatchSummary {
        &self.0.body.patch_summary
    }

    pub fn patch_body(&self) -> &BridgeCommittedPatchBody {
        &self.0.body.patch_body
    }

    pub fn digest(&self) -> &BridgeCommittedPatchDigest {
        &self.0.body.digest
    }
}
