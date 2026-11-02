use std::sync::Arc;

use super::{
    PerformedRelationalCommit, PreparedRelationalCommitCandidate, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationFailure, RelationalPublicationFailureKind,
    RelationalPublicationOutcome, StaleRelationalBranchObservation,
};
use crate::branch::{
    RelationalBranchBasisDescriptor, RelationalBranchPublicationCell,
    RelationalBranchReferenceCell, RelationalBranchRoot,
};

/// Independently borrowable Relational owner publication service.
#[derive(Debug, Clone)]
pub struct RelationalPublicationPort {
    runtime_instance_id: u64,
    configuration: crate::runtime::RelationalRuntimeConfigurationBinding,
    owner_binding: crate::runtime::RelationalRuntimeOwnerBinding,
    publication_binding: crate::runtime::RelationalRuntimePublicationBinding,
    branch_head_versions: crate::runtime::BranchHeadVersionIndexAuthority,
    history: crate::runtime::RelationalPreparationHistory,
    medium: Option<Arc<crate::durability::medium::RelationalDurableMediumBinding>>,
}

pub(crate) struct PreparedCanonicalBranchMovement {
    record_allocations: Option<crate::runtime::PendingRecordAllocations>,
    symbol_meanings: Option<crate::runtime::PreparedSymbolPublication>,
    next_cell: RelationalBranchReferenceCell,
    root: Arc<RelationalBranchRoot>,
    canonical_publication_route: crate::runtime::PreparedCanonicalPublicationRoute,
}

struct PreparedBranchPublicationPreflight {
    current_read: Option<crate::branch::AdmittedRelationalBranchBasis>,
    companion: Option<super::PreparedPublicationCompanionEffect>,
    companion_binding: Option<super::CandidateCompanionBinding>,
    movement: PreparedCanonicalBranchMovement,
    expected: RelationalBranchBasisDescriptor,
    publication_cell: RelationalBranchPublicationCell,
    next_state: Option<crate::branch::RelationalBranchReferenceMutableState>,
    next_basis: crate::branch::AdmittedRelationalBranchBasis,
    settlement_reservation: Option<crate::runtime::RelationalPendingSettlementReservation>,
    head_retirement: Option<crate::history::retention::RelationalHeadRetirementCapacity>,
    candidate_retention: Option<crate::history::retention::RelationalCandidateRetentionObligation>,
    control: crate::mvcc::RelationalOperationControl,
    expires_at: std::time::Instant,
    maximum_lifetime_millis: u64,
    branch_head_versions: crate::runtime::BranchHeadVersionIndexAuthority,
    medium: Option<Arc<crate::durability::medium::RelationalDurableMediumBinding>>,
}

impl RelationalPublicationPort {
    pub(crate) fn new(
        runtime_instance_id: u64,
        configuration: crate::runtime::RelationalRuntimeConfigurationBinding,
        owner_binding: crate::runtime::RelationalRuntimeOwnerBinding,
        publication_binding: crate::runtime::RelationalRuntimePublicationBinding,
        branch_head_versions: crate::runtime::BranchHeadVersionIndexAuthority,
        history: crate::runtime::RelationalPreparationHistory,
        medium: Option<Arc<crate::durability::medium::RelationalDurableMediumBinding>>,
    ) -> Self {
        Self {
            runtime_instance_id,
            configuration,
            owner_binding,
            publication_binding,
            branch_head_versions,
            history,
            medium,
        }
    }

    pub fn compare_and_publish(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: PreparedRelationalCommitCandidate,
    ) -> RelationalPublicationOutcome {
        self.compare_and_publish_with_codec_admission(execution, candidate, None)
    }

    pub(crate) fn compare_and_publish_checkpoint_transition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: PreparedRelationalCommitCandidate,
        admission: crate::durability::AdmittedRecoveredCodecPublication,
    ) -> RelationalPublicationOutcome {
        self.compare_and_publish_with_codec_admission(execution, candidate, Some(admission))
    }

    fn compare_and_publish_with_codec_admission(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: PreparedRelationalCommitCandidate,
        admission: Option<crate::durability::AdmittedRecoveredCodecPublication>,
    ) -> RelationalPublicationOutcome {
        match self.with_prepared_publication(
            execution,
            candidate,
            admission,
            handoff::CutoverProfile::Ordinary,
            &mut |cutover| cutover.perform(),
        ) {
            Ok(outcome) => outcome.expect("the ordinary receiver consumes its one cutover token"),
            Err(outcome) => outcome,
        }
    }
}

impl PreparedCanonicalBranchMovement {
    pub(crate) fn new(
        record_allocations: crate::runtime::PendingRecordAllocations,
        symbol_meanings: crate::runtime::PreparedSymbolPublication,
        mut next_cell: RelationalBranchReferenceCell,
        root: Arc<RelationalBranchRoot>,
        canonical_publication_route: crate::runtime::PreparedCanonicalPublicationRoute,
    ) -> Self {
        next_cell.install_root(Arc::clone(&root));
        Self {
            record_allocations: Some(record_allocations),
            symbol_meanings: Some(symbol_meanings),
            next_cell,
            root,
            canonical_publication_route,
        }
    }
}

impl crate::runtime::RelationalRuntime {
    /// The independently borrowable publication service for this runtime.
    ///
    /// [`RelationalPublicationPort::compare_and_publish`] consumes the candidate
    /// produced by
    /// [`RelationalRuntime::preparation_port`](crate::runtime::RelationalRuntime::preparation_port)
    /// and returns a typed terminal outcome rather than a `Result`, because a
    /// branch that did not move is not always an error. `Performed` transfers
    /// a settlement obligation to
    /// [`RelationalRuntime::settlement_port`](crate::runtime::RelationalRuntime::settlement_port).
    /// For a memory-only publication guarded by another owner, use
    /// [`RelationalPublicationPort::with_prepared_memory_cutover`]. Both
    /// entrances use this service's single publication engine; the scoped
    /// entrance prepares before its callback and returns actual custody after it.
    ///
    /// Different branch cells do not share an ordinary publication lock. The
    /// runnable owner workflow, including the caller-owned response to a
    /// contended patch-position reservation, is
    /// `examples/branch_local_mvcc.rs`.
    pub fn publication_port(&self) -> RelationalPublicationPort {
        RelationalPublicationPort::new(
            self.runtime_instance_id(),
            self.configuration_binding(),
            self.owner_binding(),
            self.publication_binding(),
            self.history.branch_head_version_index(),
            self.history.preparation_binding(),
            self.durability.medium(),
        )
    }

    pub fn patch_position_reservation_counters(
        &self,
    ) -> crate::runtime::RelationalPatchPositionReservationCounters {
        self.history.canonical_publication_reservation_counters()
    }
}

#[path = "port_cutover.rs"]
mod cutover;
#[path = "port_handoff.rs"]
mod handoff;
#[path = "port_preflight.rs"]
mod preflight;
pub use handoff::{PreparedMemoryRelationalCutover, ScopedRelationalPublication};
