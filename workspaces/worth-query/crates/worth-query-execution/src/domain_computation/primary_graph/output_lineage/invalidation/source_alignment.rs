use std::sync::Arc;

use im::OrdMap;
use worth_relational::facade::{
    history::CommitId, mvcc::CompanionBranchImage, publication::PatchStreamPosition,
    runtime::PositionedRelationalSnapshot,
};

use super::super::RecordedSettlementIdentity;
use super::admission::IndexAdmission;
use super::mark_state::{FullVerificationReason, MarkState, SettlementCurrentness};

pub(super) enum EqualOutputCurrentness {
    NoConsequence,
    CanonicallyEqualClean(Arc<RecordedSettlementIdentity>),
    Pending,
    FullVerificationRequired,
}

/// No history is nested inside a MarkState. Historical clearing therefore
/// retains the earlier state without building recursive history ownership.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct BranchMarkRoot {
    pub(super) retained_capacity: Option<Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>>,
    /// Shared only by replacements that retain this exact history map.
    pub(super) history_capacity: Option<Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>>,
    /// What the oldest retained version shares with versions that have left.
    pub(super) inherited_capacity: Option<Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>>,
    pub(super) current: Arc<MarkState>,
    pub(super) last_native_marking: Option<super::logical_marking::NativeMarkingReport>,
    pub(super) past: OrdMap<Option<PatchStreamPosition>, HistoricalMarkState>,
}

#[derive(Clone, Debug)]
pub(super) struct HistoricalMarkState {
    pub(super) root_id: u64,
    pub(super) commit_id: Option<CommitId>,
    pub(super) state: Arc<MarkState>,
    pub(super) next_delivery: Arc<RetainedTouchDelivery>,
}

#[derive(Debug)]
pub(super) struct RetainedTouchDelivery {
    pub(super) keys: Option<Arc<[super::fact_key::FactPostingKey]>>,
    pub(super) _capacity: Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>,
}

/// Whether a row's read basis must still be in the retained history.
#[derive(Clone, Copy, Eq, PartialEq)]
enum ReadBasis {
    Retained,
    /// The row is the origin of an authentic equality chain. Its output was
    /// proved equal to its successor while its basis was retained, so a basis
    /// that has since left the window leaves the chain's terminal to decide.
    Superseded,
}

/// Only exact native source selection can construct this observation.
pub(super) struct SnapshotAlignedMarkState<'selected> {
    state: Arc<MarkState>,
    retained: Arc<BranchMarkRoot>,
    current_root_id: u64,
    current_commit_id: Option<CommitId>,
    current_position: Option<PatchStreamPosition>,
    selected: &'selected PositionedRelationalSnapshot,
}

impl BranchMarkRoot {
    pub(super) fn initial() -> Self {
        Self {
            retained_capacity: None,
            history_capacity: None,
            inherited_capacity: None,
            current: Arc::new(MarkState::initial()),
            last_native_marking: None,
            past: OrdMap::new(),
        }
    }
}

impl<'selected> SnapshotAlignedMarkState<'selected> {
    pub(super) fn recovery_state(&self) -> Arc<MarkState> {
        Arc::clone(&self.state)
    }
    #[cfg(feature = "certification-invalidation-equivalence")]
    pub(super) fn into_full_verification_state(self) -> Arc<MarkState> {
        self.state
    }

    pub(super) fn settlement_count(&self) -> usize {
        self.state.settlements.len()
    }
    /// The caller resolves its source snapshot before reading the companion.
    /// Sparse positions from other branches are irrelevant: selection is exact,
    /// never a floor lookup or an assumption of consecutive global positions.
    pub(super) fn select_image(
        image: CompanionBranchImage<BranchMarkRoot>,
        selected: &'selected PositionedRelationalSnapshot,
    ) -> Result<Self, FullVerificationReason> {
        Self::observe_image(&image, selected)
    }

    pub(super) fn observe_image(
        image: &CompanionBranchImage<BranchMarkRoot>,
        selected: &'selected PositionedRelationalSnapshot,
    ) -> Result<Self, FullVerificationReason> {
        if image.position() == selected.position()
            && image.root_id() == selected.root_id()
            && image.commit_id() == selected.commit_id()
        {
            return Ok(Self {
                state: Arc::clone(&image.payload().current),
                retained: Arc::clone(image.payload()),
                current_root_id: image.root_id(),
                current_commit_id: image.commit_id(),
                current_position: image.position(),
                selected,
            });
        }
        let historical = image
            .payload()
            .past
            .get(&selected.position())
            .filter(|row| {
                row.root_id == selected.root_id() && row.commit_id == selected.commit_id()
            })
            .ok_or(FullVerificationReason::RetainedDeliveryGap)?;
        Ok(Self {
            state: Arc::clone(&historical.state),
            retained: Arc::clone(image.payload()),
            current_root_id: image.root_id(),
            current_commit_id: image.commit_id(),
            current_position: image.position(),
            selected,
        })
    }

    pub(super) fn currentness(
        &self,
        settlement: &RecordedSettlementIdentity,
    ) -> SettlementCurrentness<'_> {
        self.row_currentness(settlement, ReadBasis::Retained)
    }

    fn row_currentness(
        &self,
        settlement: &RecordedSettlementIdentity,
        read_basis: ReadBasis,
    ) -> SettlementCurrentness<'_> {
        let Some(row) = self.state.settlements.get(settlement) else {
            return SettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::MissingSettlement,
            );
        };
        if row.facts.for_comparison().is_none() {
            return SettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::ComputationSuperseded,
            );
        }
        let basis = &row.read_basis;
        if basis.runtime_instance_id() != self.selected.runtime_instance_id() {
            return SettlementCurrentness::Foreign;
        }
        if basis.branch_id() != self.selected.branch_id() {
            return SettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::DifferentBranch,
            );
        }
        if self.selected.position() < basis.position() {
            return SettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::BeforeReadBasis,
            );
        }
        if read_basis == ReadBasis::Superseded {
            // Authentic paired equality replaces this row's source proof.
            // Its old delivery epoch and requirements cannot veto the
            // terminal; provenance still precedes this consequence.
            return SettlementCurrentness::Clean;
        }
        let retained_at_current = basis.position() == self.current_position
            && basis.root_id() == self.current_root_id
            && basis.commit_id() == self.current_commit_id;
        let retained_in_history =
            self.retained
                .past
                .get(&basis.position())
                .is_some_and(|historical| {
                    historical.root_id == basis.root_id()
                        && historical.commit_id == basis.commit_id()
                });
        if !retained_at_current && !retained_in_history {
            return SettlementCurrentness::FullVerificationRequired(
                FullVerificationReason::RetainedDeliveryGap,
            );
        }
        if row.delivery_epoch != self.state.delivery_epoch {
            return SettlementCurrentness::FullVerificationRequired(
                self.state
                    .last_discontinuity
                    .unwrap_or(FullVerificationReason::RetainedDeliveryGap),
            );
        }
        if let Some(reason) = row.verification_requirement {
            return SettlementCurrentness::FullVerificationRequired(reason);
        }
        if !row.pending_upstream.is_empty() {
            return SettlementCurrentness::PendingUpstream(&row.pending_upstream);
        }
        if !row.dirty_ordinals.is_empty() {
            return SettlementCurrentness::Dirty(&row.dirty_ordinals);
        }
        SettlementCurrentness::Clean
    }

    /// Resolve an exact consumed identity through the actor's versioned
    /// stable-equality chain. The old row stays dirty; only output evidence
    /// may consume a clean certified successor consequence.
    pub(super) fn equal_output_currentness(
        &self,
        identity: &RecordedSettlementIdentity,
        admission: &mut impl IndexAdmission,
    ) -> Result<EqualOutputCurrentness, worth_relational::facade::mvcc::CompanionPreflightStop>
    {
        admission.ordered_read(self.state.equal_links.len())?;
        let Some(first) = self.state.equal_links.get(identity) else {
            return Ok(EqualOutputCurrentness::NoConsequence);
        };
        if first.next.is_none() {
            return Ok(EqualOutputCurrentness::NoConsequence);
        }
        admission.ordered_read(self.state.settlements.len())?;
        admission.ordered_read(self.retained.past.len())?;
        if let SettlementCurrentness::FullVerificationRequired(_) | SettlementCurrentness::Foreign =
            self.row_currentness(identity, ReadBasis::Superseded)
        {
            return Ok(EqualOutputCurrentness::FullVerificationRequired);
        }
        let mut current = identity;
        let mut latest = None;
        let mut hops = 0usize;
        loop {
            admission.work(1)?;
            hops = hops.checked_add(1).ok_or(
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
            )?;
            if hops > self.state.equal_links.len() {
                return Ok(EqualOutputCurrentness::FullVerificationRequired);
            }
            admission.ordered_read(self.state.equal_links.len())?;
            let Some(link) = self.state.equal_links.get(current) else {
                return Ok(EqualOutputCurrentness::FullVerificationRequired);
            };
            let Some(next) = &link.next else {
                admission.ordered_read(self.state.settlements.len())?;
                admission.ordered_read(self.retained.past.len())?;
                return Ok(match self.currentness(current) {
                    SettlementCurrentness::Clean => EqualOutputCurrentness::CanonicallyEqualClean(
                        latest.expect("a certified equality has a successor"),
                    ),
                    SettlementCurrentness::Dirty(_) | SettlementCurrentness::PendingUpstream(_) => {
                        EqualOutputCurrentness::Pending
                    }
                    SettlementCurrentness::FullVerificationRequired(_)
                    | SettlementCurrentness::Foreign => {
                        EqualOutputCurrentness::FullVerificationRequired
                    }
                });
            };
            admission.ordered_read(self.state.equal_links.len())?;
            if self
                .state
                .equal_links
                .get(next)
                .is_none_or(|following| following.prior.as_deref() != Some(current))
            {
                return Ok(EqualOutputCurrentness::FullVerificationRequired);
            }
            latest = Some(Arc::clone(next));
            current = next;
        }
    }
}
