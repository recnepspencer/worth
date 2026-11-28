use std::sync::Arc;

use im::OrdMap;
use im::OrdSet;
use worth_relational::facade::history::CommitId;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;

use super::super::RecordedSettlementIdentity;
use super::fact_key::FactPostingKey;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FactPosting {
    pub settlement: Arc<RecordedSettlementIdentity>,
    pub ordinal: usize,
}

/// Why a retained row's first reader compares it in full. Each value is a
/// reason a row can record. A stop ends the request that meets it instead,
/// so none is here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FullVerificationReason {
    NativeRevisionUnavailable,
    NativeFactRevisionUnavailable(usize),
    IndexedSelectionDenied(crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial),
    IndexedSelectionFactDenied(usize, crate::domain_computation::primary_graph::application_attempt::IndexedSelectionReobserveDenial),
    UnsupportedFact,
    CheckpointRestore,
    DifferentBranch,
    BeforeReadBasis,
    RetainedDeliveryGap,
    MissingSettlement,
    DeclaredChangeUnavailable,
    /// A stop ended this row's registration after its World effect, which
    /// stays authoritative, so the row holds no complete marks.
    RegistrationIncomplete,
    /// Original computation inputs were changed by its own publication.
    ComputationSuperseded,
}

impl FullVerificationReason {
    /// No mark row answers for the settlement at the selected source: there
    /// is none, it is still in its checkpoint posture, or it was read on
    /// another branch or at a later basis. A foreign source is the one other
    /// such case, and it is a stop, not a reason.
    pub const fn no_row_answers(self) -> bool {
        matches!(
            self,
            Self::CheckpointRestore
                | Self::MissingSettlement
                | Self::DifferentBranch
                | Self::BeforeReadBasis
        )
    }
}

/// Derived continuity identity scoped by the owning source and branch. The
/// native commit that declared a discontinuity replaces a redundant counter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveryEpoch(Option<CommitId>);

impl DeliveryEpoch {
    pub const fn initial() -> Self {
        Self(None)
    }

    pub const fn after_discontinuity(commit: CommitId) -> Self {
        Self(Some(commit))
    }

    pub const fn commit_id(self) -> Option<CommitId> {
        self.0
    }
}

/// Complete delivery is a branch-local epoch. A declared unavailable change
/// advances it once without scanning settlements; individual native verification
/// can then re-establish a row in that exact epoch.
#[derive(Clone, Debug)]
pub struct SettlementMarks {
    /// Changes since retained computation, preserved by ordinary dirty clearing.
    pub computation_frontier: Option<OrdSet<usize>>,
    pub work_membership: Option<Arc<RequiredWorkMembership>>,

    pub posting_projection: Arc<super::settlement::PostingProjection>,
    pub consumed_upstream: OrdSet<Arc<RecordedSettlementIdentity>>,
    pub facts: super::super::RetainedSourceFacts,
    pub output_facts: Option<super::output_facts::RegisteredOutputFacts>,
    pub output_coverage: OutputFactCoverage,
    pub read_basis: Arc<PositionedRelationalSnapshot>,
    pub delivery_epoch: DeliveryEpoch,
    pub dirty_ordinals: OrdSet<usize>,
    pub pending_upstream: OrdSet<Arc<RecordedSettlementIdentity>>,
    pub verification_requirement: Option<FullVerificationReason>,
    /// A newer settlement of the same demand replaced this row. It leaves the
    /// live root once no live row or equality reader reaches it.
    pub superseded: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFactCoverage {
    Absent,
    Performed,
    Stable,
}

impl OutputFactCoverage {
    pub const fn complete(self) -> bool {
        !matches!(self, Self::Absent)
    }
}

impl SettlementMarks {
    pub fn fact_at(&self, ordinal: usize) -> Option<&WorthQueryApplicationObservedFact> {
        self.facts.for_comparison()?;
        if let Some(fact) = self.facts.postconditions().get(ordinal) {
            return Some(fact);
        }
        let output = self.output_facts.as_ref()?;
        let fact = output.facts.get(ordinal.checked_sub(self.facts.len())?)?;
        Some(fact)
    }
}

/// One certified equality chain per exact output identity. Both directions
/// live in the same versioned mark root, so old source images retain their
/// prior consequence and future dirty delivery can reach old consumers.
#[derive(Clone, Debug)]
pub struct EqualOutputLink {
    pub prior: Option<Arc<RecordedSettlementIdentity>>,
    pub next: Option<Arc<RecordedSettlementIdentity>>,
}

#[derive(Clone, Debug)]
pub struct MarkState {
    pub source_publications: OrdMap<Arc<crate::domain_computation::primary_graph::application_contribution::OutputSourceAddress>, Arc<super::output_availability::SourcePublicationState>>,
    pub source_publication_postings: OrdMap<super::output_availability::PublicationSourceKey, OrdSet<Arc<crate::domain_computation::primary_graph::application_contribution::OutputSourceAddress>>>,
    pub source_publication_posting_count: usize,
    pub source_publication_head_count: usize,

    pub posting_count: usize,

    pub next_posting_projection: u64,
    pub posting_bindings: OrdMap<u64, Arc<super::settlement::PostingBindings>>,
    /// Whole-index upper bound for carried basis allocations. Registration's
    /// initial basis has its own fact ticket; a carried clone is index-owned.

    pub downstream_edge_count: usize,
    pub dirty_ordinal_count: usize,
    pub computation_dirty_ordinal_count: usize,
    pub pending_edge_count: usize,
    pub delivery_epoch: DeliveryEpoch,
    pub last_discontinuity: Option<FullVerificationReason>,
    pub postings: OrdMap<Arc<FactPostingKey>, OrdSet<u64>>,
    pub settlements: OrdMap<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>,
    pub downstream:
        OrdMap<Arc<RecordedSettlementIdentity>, OrdSet<Arc<RecordedSettlementIdentity>>>,
    pub equal_links: OrdMap<Arc<RecordedSettlementIdentity>, Arc<EqualOutputLink>>,
}

pub enum SettlementCurrentness<'state> {
    Clean,
    Dirty(&'state OrdSet<usize>),
    PendingUpstream(&'state OrdSet<Arc<RecordedSettlementIdentity>>),
    FullVerificationRequired(FullVerificationReason),
    /// The row was read in another runtime. Its marks say nothing here.
    Foreign,
}

impl MarkState {
    /// Reserve the next derived posting identity before any index installation.
    pub fn reserve_posting_projection(
        &mut self,
    ) -> Result<u64, worth_relational::facade::mvcc::CompanionPreflightStop> {
        let id = self.next_posting_projection;
        self.next_posting_projection = id.checked_add(1)
            .ok_or(worth_relational::facade::mvcc::CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        Ok(id)
    }

    pub fn initial() -> Self {
        Self {
            source_publications: OrdMap::new(),
            source_publication_postings: OrdMap::new(),
            source_publication_posting_count: 0,
            source_publication_head_count: 0,

            posting_count: 0,

            next_posting_projection: 0,
            posting_bindings: OrdMap::new(),

            downstream_edge_count: 0,
            dirty_ordinal_count: 0,
            computation_dirty_ordinal_count: 0,
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

#[cfg(test)]
mod projection_identity_tests {
    use super::*;
    #[test]
    fn projection_identity_overflow_refuses_without_reissuing_an_identity() {
        let mut state = MarkState::initial();
        state.next_posting_projection = u64::MAX - 1;
        assert_eq!(state.reserve_posting_projection(), Ok(u64::MAX - 1));
        assert_eq!(state.reserve_posting_projection(), Err(worth_relational::facade::mvcc::CompanionPreflightStop::PreparationMemoryCounterOverflow));
        assert_eq!(state.next_posting_projection, u64::MAX);
    }
}

mod court_doors_for_test;
