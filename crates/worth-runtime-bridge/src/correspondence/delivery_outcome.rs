use std::sync::Arc;

use crate::facade::{
    BridgeCommittedRecordChange, BridgeSemanticAspectChange, RelationalBridgeRecordIdentityParts,
    TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity, TruthSnapshotIdentity,
};

use super::{
    BridgeAdmittedTruthCommitIdentity, BridgeAdmittedTruthRecordIdentity,
    BridgeAdmittedTruthSnapshotIdentity, BridgeCorrespondenceBasis, BridgeCorrespondenceDenialKind,
    BridgeSemanticDependencyCandidate, CorrespondenceDeliveryCounters,
};

/// One authoritative lower-runtime change actually admitted by an installed
/// semantic correspondence. Construction remains Bridge-owned so consumers
/// cannot promote a raw envelope item into delivered-change authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeDeliveredCorrespondenceChange {
    inner: BridgeDeliveredCorrespondenceChangeInner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BridgeDeliveredCorrespondenceChangeInner {
    SemanticAspect {
        entity_identity: Arc<str>,
        relational_record_identity: Option<RelationalBridgeRecordIdentityParts>,
        change: BridgeSemanticAspectChange,
    },
    StructuralRecord(BridgeCommittedRecordChange),
}

impl BridgeDeliveredCorrespondenceChange {
    pub(crate) fn semantic_aspect(
        entity_identity: Arc<str>,
        relational_record_identity: Option<RelationalBridgeRecordIdentityParts>,
        change: BridgeSemanticAspectChange,
    ) -> Self {
        Self {
            inner: BridgeDeliveredCorrespondenceChangeInner::SemanticAspect {
                entity_identity,
                relational_record_identity,
                change,
            },
        }
    }

    pub(crate) fn structural_record(change: BridgeCommittedRecordChange) -> Self {
        Self {
            inner: BridgeDeliveredCorrespondenceChangeInner::StructuralRecord(change),
        }
    }

    pub fn semantic_change(&self) -> Option<&BridgeSemanticAspectChange> {
        match &self.inner {
            BridgeDeliveredCorrespondenceChangeInner::SemanticAspect { change, .. } => Some(change),
            BridgeDeliveredCorrespondenceChangeInner::StructuralRecord(_) => None,
        }
    }

    pub fn structural_change(&self) -> Option<&BridgeCommittedRecordChange> {
        match &self.inner {
            BridgeDeliveredCorrespondenceChangeInner::SemanticAspect { .. } => None,
            BridgeDeliveredCorrespondenceChangeInner::StructuralRecord(change) => Some(change),
        }
    }

    /// Returns the binding-aware semantic kind Bridge admitted for this exact
    /// delivered change. Downstream indexes may use it to select candidates,
    /// but the returned kind is not independent authority.
    pub fn effective_change_kind_for(
        &self,
        dependency: &BridgeSemanticDependencyCandidate,
    ) -> Option<worth_foundational::facade::AuthoritativeAspectChangeKind> {
        match &self.inner {
            BridgeDeliveredCorrespondenceChangeInner::SemanticAspect { change, .. } => {
                Some(change.kind())
            }
            BridgeDeliveredCorrespondenceChangeInner::StructuralRecord(change) => {
                super::semantic_delivery_match::structural_change_kind(
                    dependency.binding(),
                    change.kind(),
                )
            }
        }
    }

    pub fn entity_identity(&self) -> Option<&str> {
        match &self.inner {
            BridgeDeliveredCorrespondenceChangeInner::SemanticAspect {
                entity_identity, ..
            } => Some(entity_identity),
            BridgeDeliveredCorrespondenceChangeInner::StructuralRecord(_) => None,
        }
    }

    pub fn relational_record_identity(&self) -> Option<RelationalBridgeRecordIdentityParts> {
        match &self.inner {
            BridgeDeliveredCorrespondenceChangeInner::SemanticAspect {
                relational_record_identity,
                ..
            } => *relational_record_identity,
            BridgeDeliveredCorrespondenceChangeInner::StructuralRecord(change) => {
                Some(change.record_identity())
            }
        }
    }

    pub fn admitted_record_identity(&self) -> Option<BridgeAdmittedTruthRecordIdentity> {
        self.relational_record_identity()
            .map(BridgeAdmittedTruthRecordIdentity::admit)
    }
}

/// The exact owner-delivered change set after correspondence admission and
/// Signal mutation. This preserves the Bridge decision instead of asking Query
/// to reinterpret the committed envelope.
#[derive(Debug, Clone)]
pub struct BridgeDeliveredCorrespondenceChangeSet {
    basis: BridgeCorrespondenceBasis,
    dependency: BridgeSemanticDependencyCandidate,
    commit_identity: BridgeAdmittedTruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: BridgeAdmittedTruthSnapshotIdentity,
    branch_identity: TruthBranchIdentity,
    changes: Vec<BridgeDeliveredCorrespondenceChange>,
}

impl BridgeDeliveredCorrespondenceChangeSet {
    pub(crate) fn new(
        basis: BridgeCorrespondenceBasis,
        dependency: BridgeSemanticDependencyCandidate,
        envelope: &crate::input::envelope::BridgeCommittedPatchEnvelope,
        changes: Vec<BridgeDeliveredCorrespondenceChange>,
    ) -> Self {
        Self {
            basis,
            dependency,
            commit_identity: BridgeAdmittedTruthCommitIdentity::admit(
                envelope.commit_identity().clone(),
            ),
            patch_identity: envelope.patch_identity().clone(),
            snapshot_identity: BridgeAdmittedTruthSnapshotIdentity::admit(
                envelope.snapshot_identity().clone(),
            ),
            branch_identity: envelope.branch_identity().clone(),
            changes,
        }
    }

    pub fn basis(&self) -> &BridgeCorrespondenceBasis {
        &self.basis
    }

    pub fn dependency(&self) -> &BridgeSemanticDependencyCandidate {
        &self.dependency
    }

    pub fn commit_identity(&self) -> &TruthCommitIdentity {
        self.commit_identity.projection()
    }

    pub fn admitted_commit_identity(&self) -> &BridgeAdmittedTruthCommitIdentity {
        &self.commit_identity
    }

    pub fn patch_identity(&self) -> &TruthPatchIdentity {
        &self.patch_identity
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        self.snapshot_identity.projection()
    }

    pub fn admitted_snapshot_identity(&self) -> &BridgeAdmittedTruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn retains_delivery_identity(
        &self,
        commit_identity: &TruthCommitIdentity,
        snapshot_identity: &TruthSnapshotIdentity,
    ) -> bool {
        self.commit_identity.projection() == commit_identity
            && self.snapshot_identity.projection() == snapshot_identity
    }

    pub fn branch_identity(&self) -> &TruthBranchIdentity {
        &self.branch_identity
    }

    pub fn changes(&self) -> &[BridgeDeliveredCorrespondenceChange] {
        &self.changes
    }

    pub(crate) fn retains_same_delivery_as(&self, other: &Self) -> bool {
        self.basis == other.basis
            && self.dependency == other.dependency
            && self.commit_identity.projection() == other.commit_identity.projection()
            && self.patch_identity == other.patch_identity
            && self.snapshot_identity.projection() == other.snapshot_identity.projection()
            && self.branch_identity == other.branch_identity
            && self.changes == other.changes
    }
}

#[derive(Debug, Clone)]
pub struct BridgeCorrespondenceDeliveryReceipt {
    counters: CorrespondenceDeliveryCounters,
    truth_change: super::BridgeDeliveredTruthChange,
    prepared_signal: Option<super::BridgePreparedScopedSignalInvalidation>,
    conditional_transition:
        Option<worth_signal::facade::branch::SignalConditionalSuccessorTransition>,
}

impl BridgeCorrespondenceDeliveryReceipt {
    pub(crate) const fn new(
        counters: CorrespondenceDeliveryCounters,
        change_set: BridgeDeliveredCorrespondenceChangeSet,
        prepared_signal: Option<super::BridgePreparedScopedSignalInvalidation>,
    ) -> Self {
        Self {
            counters,
            truth_change: super::BridgeDeliveredTruthChange::new(change_set),
            prepared_signal,
            conditional_transition: None,
        }
    }

    pub(crate) fn with_conditional_transition(
        mut self,
        transition: worth_signal::facade::branch::SignalConditionalSuccessorTransition,
    ) -> Self {
        self.conditional_transition = Some(transition);
        self
    }

    pub(crate) const fn conditional_transition(
        &self,
    ) -> Option<&worth_signal::facade::branch::SignalConditionalSuccessorTransition> {
        self.conditional_transition.as_ref()
    }

    pub const fn counters(&self) -> CorrespondenceDeliveryCounters {
        self.counters
    }

    pub const fn change_set(&self) -> &BridgeDeliveredCorrespondenceChangeSet {
        self.truth_change.change_set()
    }

    pub const fn truth_change(&self) -> &super::BridgeDeliveredTruthChange {
        &self.truth_change
    }

    pub const fn prepared_signal_invalidation(
        &self,
    ) -> Option<&super::BridgePreparedScopedSignalInvalidation> {
        self.prepared_signal.as_ref()
    }

    pub const fn truth_targets_admitted(&self) -> usize {
        self.counters.truth_targets_admitted()
    }

    pub const fn source_load_attempts(&self) -> usize {
        self.counters.source_load_attempts()
    }

    pub const fn source_envelopes_loaded(&self) -> usize {
        self.counters.source_envelopes_loaded()
    }

    pub const fn allocation_registry_lock_attempts(&self) -> usize {
        self.counters.allocation_registry_lock_attempts()
    }

    pub const fn signal_capability_admissions(&self) -> usize {
        self.counters.signal_capability_admissions()
    }

    pub const fn failed_deliveries(&self) -> usize {
        self.counters.failed_deliveries()
    }

    pub const fn correspondence_lookups(&self) -> usize {
        self.counters.correspondence_lookups()
    }

    pub const fn signal_seeds_emitted(&self) -> usize {
        self.counters.signal_seeds_emitted()
    }

    pub const fn node_fan_out(&self) -> usize {
        self.counters.node_fan_out()
    }

    pub const fn slots_touched(&self) -> usize {
        self.counters.slots_touched()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeCorrespondenceDeliveryDenial {
    kind: BridgeCorrespondenceDenialKind,
    counters: CorrespondenceDeliveryCounters,
}

impl BridgeCorrespondenceDeliveryDenial {
    pub(crate) const fn new(
        kind: BridgeCorrespondenceDenialKind,
        counters: CorrespondenceDeliveryCounters,
    ) -> Self {
        Self { kind, counters }
    }

    pub const fn kind(self) -> BridgeCorrespondenceDenialKind {
        self.kind
    }

    pub const fn counters(self) -> CorrespondenceDeliveryCounters {
        self.counters
    }
}
