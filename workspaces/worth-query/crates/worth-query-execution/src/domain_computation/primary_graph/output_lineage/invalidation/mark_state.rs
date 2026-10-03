use std::sync::Arc;

use im::{OrdMap, OrdSet};
use worth_relational::facade::{history::CommitId, runtime::PositionedRelationalSnapshot};

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;

use super::super::RecordedSettlementIdentity;
use super::fact_key::FactPostingKey;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct FactPosting {
    pub(super) settlement: Arc<RecordedSettlementIdentity>,
    pub(super) ordinal: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum FullVerificationReason {
    NativeRevisionUnavailable,
    UnsupportedFact,
    CheckpointRestore,
    ForeignSource,
    DifferentBranch,
    BeforeReadBasis,
    RetainedDeliveryGap,
    MissingSettlement,
    IndexDefinitionChanged,
    AspectContractChanged,
    ProgramSchemaChanged,
    DeclaredChangeUnavailable,
    MarkingAdmissionDenied(worth_relational::facade::mvcc::CompanionPreflightStop),
    DerivedEditPending(worth_relational::facade::mvcc::CompanionCellEditStop),
    SelectedSourceUnavailable(worth_relational::facade::runtime::SnapshotPositionDenial),
}

/// Derived continuity identity scoped by the owning source and branch. The
/// native commit that declared a discontinuity replaces a redundant counter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DeliveryEpoch(Option<CommitId>);

impl DeliveryEpoch {
    pub(super) const fn initial() -> Self {
        Self(None)
    }

    pub(super) const fn after_discontinuity(commit: CommitId) -> Self {
        Self(Some(commit))
    }

    pub(super) const fn commit_id(self) -> Option<CommitId> {
        self.0
    }
}

/// Complete delivery is a branch-local epoch. A declared unavailable change
/// advances it once without scanning settlements; individual native verification
/// can then re-establish a row in that exact epoch.
#[derive(Clone, Debug)]
pub(super) struct SettlementMarks {
    pub(super) work_membership: Option<Arc<RequiredWorkMembership>>,
    pub(super) _fact_capacity: Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>,
    pub(super) posting_payload_bytes: u64,
    pub(super) posting_ordinals: OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
    pub(super) consumed_upstream: OrdSet<Arc<RecordedSettlementIdentity>>,
    pub(super) facts: Arc<[WorthQueryApplicationObservedFact]>,
    pub(super) output_facts: Option<super::output_facts::RegisteredOutputFacts>,
    pub(super) output_coverage: OutputFactCoverage,
    pub(super) read_basis: Arc<PositionedRelationalSnapshot>,
    pub(super) delivery_epoch: DeliveryEpoch,
    pub(super) dirty_ordinals: OrdSet<usize>,
    pub(super) pending_upstream: OrdSet<Arc<RecordedSettlementIdentity>>,
    pub(super) verification_requirement: Option<FullVerificationReason>,
    /// A newer settlement of the same demand replaced this row. It leaves the
    /// live root once no live row or equality reader reaches it.
    pub(super) superseded: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OutputFactCoverage {
    Absent,
    Performed,
    Stable,
}

impl OutputFactCoverage {
    pub(super) const fn complete(self) -> bool {
        !matches!(self, Self::Absent)
    }
}

impl SettlementMarks {
    pub(super) fn fact_at(&self, ordinal: usize) -> Option<&WorthQueryApplicationObservedFact> {
        self.facts.get(ordinal).or_else(|| {
            self.output_facts
                .as_ref()?
                .facts
                .get(ordinal.checked_sub(self.facts.len())?)
        })
    }
}

/// One certified equality chain per exact output identity. Both directions
/// live in the same versioned mark root, so old source images retain their
/// prior consequence and future dirty delivery can reach old consumers.
#[derive(Clone, Debug)]
pub(super) struct EqualOutputLink {
    pub(super) prior: Option<Arc<RecordedSettlementIdentity>>,
    pub(super) next: Option<Arc<RecordedSettlementIdentity>>,
}

#[derive(Clone, Debug)]
pub(super) struct MarkState {
    pub(super) retained_capacity: Option<Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>>,
    pub(super) posting_count: usize,
    pub(super) key_payload_bytes: u64,
    pub(super) settlement_key_payload_bytes: u64,
    pub(super) downstream_edge_count: usize,
    pub(super) dirty_ordinal_count: usize,
    pub(super) pending_edge_count: usize,
    pub(super) delivery_epoch: DeliveryEpoch,
    pub(super) last_discontinuity: Option<FullVerificationReason>,
    pub(super) postings: OrdMap<Arc<FactPostingKey>, OrdSet<FactPosting>>,
    pub(super) settlements: OrdMap<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>,
    pub(super) downstream:
        OrdMap<Arc<RecordedSettlementIdentity>, OrdSet<Arc<RecordedSettlementIdentity>>>,
    pub(super) equal_links: OrdMap<Arc<RecordedSettlementIdentity>, Arc<EqualOutputLink>>,
}

pub(super) enum SettlementCurrentness<'state> {
    Clean,
    Dirty(&'state OrdSet<usize>),
    PendingUpstream(&'state OrdSet<Arc<RecordedSettlementIdentity>>),
    FullVerificationRequired(FullVerificationReason),
}

impl MarkState {
    pub(super) fn initial() -> Self {
        Self {
            retained_capacity: None,
            posting_count: 0,
            key_payload_bytes: 0,
            settlement_key_payload_bytes: 0,
            downstream_edge_count: 0,
            dirty_ordinal_count: 0,
            pending_edge_count: 0,
            delivery_epoch: DeliveryEpoch::initial(),
            last_discontinuity: None,
            postings: OrdMap::new(),
            settlements: OrdMap::new(),
            downstream: OrdMap::new(),
            equal_links: OrdMap::new(),
        }
    }
}
