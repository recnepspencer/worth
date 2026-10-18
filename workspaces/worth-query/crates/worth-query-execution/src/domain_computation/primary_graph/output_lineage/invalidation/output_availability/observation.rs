//! Native comparison of a retained source slot; no scheduling or producer work.

use std::{fmt, sync::Arc};

use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

use super::{ObservedSourcePublication, OutputSourceAddress, SourceInvalidationOwner};
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Stop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

/// Sealed owner port prevents a retained fact from owning its native index
/// recursively. Only the original invalidation owner implements or issues it.
pub(super) trait SourcePublicationReader: Send + Sync {
    fn compare(
        &self,
        observed: &ObservedSourcePublication,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        positioned: &PositionedRelationalSnapshot,
        maximum_work: usize,
    ) -> Result<(bool, usize), Stop>;
}

pub(super) fn bind_owner(
    owner: &Arc<SourceInvalidationOwner>,
) -> std::sync::Weak<dyn SourcePublicationReader> {
    let port: Arc<dyn SourcePublicationReader> = owner.clone();
    Arc::downgrade(&port)
}

impl fmt::Debug for ObservedSourcePublication {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output
            .debug_struct("ObservedSourcePublication")
            .field("address", &self.address)
            .field("selected", &self.selected)
            .finish_non_exhaustive()
    }
}

impl PartialEq for ObservedSourcePublication {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
            && self.posture() == other.posture()
            && self.source_epoch == other.source_epoch
            && Arc::ptr_eq(&self.state, &other.state)
    }
}

impl Eq for ObservedSourcePublication {}

impl ObservedSourcePublication {
    pub(in crate::domain_computation::primary_graph) fn address(&self) -> Arc<OutputSourceAddress> {
        Arc::clone(&self.address)
    }

    pub(in crate::domain_computation::primary_graph) fn posture(
        &self,
    ) -> crate::domain_computation::primary_graph::invariant_projection::ObservedOutputPosture {
        self.candidate.as_ref().map_or(crate::domain_computation::primary_graph::invariant_projection::ObservedOutputPosture::Pending, |c| c.expected())
    }
    pub(in crate::domain_computation::primary_graph) fn touches_entity(
        &self,
        entity: worth_relational::facade::identity::EntityId,
    ) -> bool {
        self.source_facts()
            .iter()
            .any(|fact| fact.touches_entity(entity))
    }

    pub(in crate::domain_computation::primary_graph) fn source_epoch(&self) -> [u8; 32] {
        self.source_epoch.bytes()
    }

    pub(in crate::domain_computation::primary_graph) fn completed_heads(
        &self,
    ) -> impl ExactSizeIterator<
        Item = (
            &str,
            [u8; 32],
            worth_runtime_world::facade::ProductBranchIncarnation,
            u64,
            usize,
        ),
    > {
        self.state.heads.iter().map(|head| {
            let (occurrence, generation, slot) = head.identity.address();
            (
                head.key.source.output_binding.as_str(),
                head.key.partition,
                occurrence,
                generation,
                slot,
            )
        })
    }

    pub(in crate::domain_computation::primary_graph) fn posting_facts(
        &self,
    ) -> impl Iterator<Item = &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>
    {
        self.read_closure
            .facts()
            .iter()
            .chain(self.read_closure.authorization_facts())
            .chain(
                self.candidate
                    .iter()
                    .flat_map(|candidate| candidate.posting_facts()),
            )
    }

    pub(in crate::domain_computation::primary_graph) fn source_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        self.read_closure.facts()
    }

    /// Compare only a native snapshot of this original source/branch. An
    /// evicted or unaligned slot is unavailable; absence never means Pending.
    pub(in crate::domain_computation::primary_graph) fn remains_equal(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        maximum_work: usize,
    ) -> Result<(bool, usize), crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure>{
        use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Stop;
        let positioned = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| Stop::Unavailable)?;
        if positioned.runtime_instance_id() != self.selected.runtime_instance_id()
            || positioned.branch_id() != self.selected.branch_id()
        {
            return Err(Stop::Unavailable);
        }
        self.owner.upgrade().ok_or(Stop::Unavailable)?.compare(
            self,
            runtime,
            execution,
            snapshot,
            &positioned,
            maximum_work,
        )
    }
}

impl SourcePublicationReader for SourceInvalidationOwner {
    fn compare(
        &self,
        observed: &ObservedSourcePublication,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        positioned: &PositionedRelationalSnapshot,
        maximum_work: usize,
    ) -> Result<(bool, usize), Stop> {
        use super::super::admission::IndexAdmission;
        let mut admission = self.edit_admission_within(
            std::num::NonZeroUsize::new(maximum_work).ok_or(Stop::WorkBudgetExceeded)?,
        );
        let cell = self.cell_for_read(positioned, &mut admission)
            .map_err(|stop| Stop::OutputObservation(crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop(stop)))?
            .ok_or(Stop::Unavailable)?;
        let image = cell.read_image();
        admission
            .ordered_read(image.payload().past.len())
            .map_err(|_| Stop::WorkBudgetExceeded)?;
        let state = if image.root_id() == positioned.root_id()
            && image.commit_id() == positioned.commit_id()
            && image.position() == positioned.position()
        {
            &image.payload().current
        } else {
            &image
                .payload()
                .past
                .get(&positioned.position())
                .filter(|past| {
                    past.root_id == positioned.root_id() && past.commit_id == positioned.commit_id()
                })
                .ok_or(Stop::Unavailable)?
                .state
        };
        let work =
            super::super::index_capacity::ordered_navigation_work(state.source_publications.len())
                .and_then(|work| work.checked_mul(8 + 4 * 32))
                .ok_or(Stop::WorkBudgetExceeded)?;
        admission
            .charge_external_work(work)
            .map_err(|_| Stop::WorkBudgetExceeded)?;
        let actual = state
            .source_publications
            .get(&observed.address)
            .ok_or(Stop::Unavailable)?;
        let mut equal = Arc::ptr_eq(actual, &observed.state);
        if equal {
            observed
                .read_closure
                .validate_for_publication(execution, runtime, snapshot, &mut admission)
                .map_err(Stop::OutputObservation)?;
            equal = observed.read_closure.source_facts_current(
                execution,
                runtime,
                snapshot,
                &mut admission,
            )?;
        }
        if equal && let Some(candidate) = &observed.candidate {
            equal = candidate
                .remains_in(
                    execution,
                    self,
                    runtime,
                    snapshot,
                    positioned,
                    &mut admission,
                )
                .map_err(Stop::OutputObservation)?;
        }
        Ok((
            equal,
            usize::try_from(admission.charged_work()).map_err(|_| Stop::WorkBudgetExceeded)?,
        ))
    }
}
