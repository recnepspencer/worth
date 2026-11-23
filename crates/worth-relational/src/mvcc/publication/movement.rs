//! Compare the selected branch state and consume one prepared movement.
use std::sync::Arc;

mod preflight;

use super::current_read::PublicationReadCoordination;
use super::{
    CandidateCompanionBinding, PreparedPublicationCompanionEffect, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationFailure, RelationalPublicationFailureKind,
    RelationalPublicationOutcome, StaleRelationalBranchObservation,
};
use crate::branch::reference::RelationalBranchReferenceCell;
use crate::branch::root::RelationalBranchRoot;
use crate::history::retention::TransferredHeadRetirement;

pub(super) type PerformedBranchMovement = (
    Arc<crate::history::data::PositionedCanonicalCommit>,
    TransferredHeadRetirement,
);

pub(super) struct BranchMovementCutover<'a, 'cell> {
    pub(super) coordination: &'a PublicationReadCoordination<'cell>,
    pub(super) publication_cell:
        &'a crate::branch::reference_publication_cell::RelationalBranchPublicationCell,
    pub(super) expected: &'a crate::branch::basis::RelationalBranchBasisDescriptor,
    pub(super) next_state:
        &'a mut Option<crate::branch::reference::RelationalBranchReferenceMutableState>,
    pub(super) head_retirement:
        &'a mut Option<crate::history::retention::RelationalHeadRetirementCapacity>,
    pub(super) companion: Option<&'a mut PreparedPublicationCompanionEffect>,
    pub(super) companion_binding: Option<&'a CandidateCompanionBinding>,
    pub(super) medium:
        Option<&'a crate::durability::published_medium::RelationalDurableMediumBinding>,
    pub(super) branch_head_versions:
        &'a crate::runtime::state::subsystems::history::BranchHeadVersionIndexAuthority,
}

pub(crate) struct PreparedCanonicalBranchMovement {
    pub(super) record_allocations: Option<crate::runtime::state::subsystems::record_identity::PendingRecordAllocations>,
    pub(super) symbol_meanings: Option<crate::runtime::state::subsystems::services::symbol_table::PreparedSymbolPublication>,
    next_cell: RelationalBranchReferenceCell,
    pub(super) root: Arc<RelationalBranchRoot>,
    pub(super) canonical_publication_route: crate::runtime::state::subsystems::history::canonical_publication_routes::PreparedCanonicalPublicationRoute,
}

impl PreparedCanonicalBranchMovement {
    pub(crate) fn new(
        record_allocations: crate::runtime::state::subsystems::record_identity::PendingRecordAllocations,
        symbol_meanings: crate::runtime::state::subsystems::services::symbol_table::PreparedSymbolPublication,
        mut next_cell: RelationalBranchReferenceCell,
        root: Arc<RelationalBranchRoot>,
        canonical_publication_route: crate::runtime::state::subsystems::history::canonical_publication_routes::PreparedCanonicalPublicationRoute,
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

impl BranchMovementCutover<'_, '_> {
    pub(super) fn compare_and_move(
        &mut self,
        route: &crate::runtime::state::subsystems::history::canonical_publication_routes::PreparedCanonicalPublicationRoute,
        next_root: &Arc<RelationalBranchRoot>,
        symbol_meanings: &mut Option<
            crate::runtime::state::subsystems::services::symbol_table::PreparedSymbolPublication,
        >,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        deadline: Option<std::time::Instant>,
        mut allow_effect: Option<&mut dyn FnMut() -> bool>,
        on_reservation: impl FnOnce(),
        settlement: &mut crate::runtime::state::runtime_state::publication_settlement_registry::RelationalPendingSettlementReservation,
    ) -> Result<PerformedBranchMovement, Option<RelationalPublicationOutcome>> {
        let (mut publication_state, read_state) = match self.coordination.enter_states() {
            Ok(states) => states,
            Err(denial) => {
                return Err(Some(RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::CurrentReadDependency(denial),
                )));
            }
        };
        match publication_state.lifecycle_posture() {
            crate::branch::lifecycle::RelationalBranchLifecyclePosture::Live => {}
            crate::branch::lifecycle::RelationalBranchLifecyclePosture::Archived => {
                return Err(Some(RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::Archived,
                )));
            }
            crate::branch::lifecycle::RelationalBranchLifecyclePosture::Deleting => {
                return Err(Some(RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::Deleting,
                )));
            }
        }
        let observed_cell = publication_state.snapshot_cell();
        let Some(observed_root) = observed_cell.root() else {
            return Err(Some(RelationalPublicationOutcome::failed(
                RelationalPublicationFailure::new(
                    RelationalPublicationFailureKind::SelectedRootUnavailable,
                    "selected branch root is unavailable before publication movement",
                ),
            )));
        };
        let observed = match crate::branch::basis_issuance::descriptor_for_cell(
            &observed_cell,
            &observed_root,
        ) {
            Ok(descriptor) => descriptor,
            Err(denial) => {
                return Err(Some(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::BranchObservation(denial.clone()),
                        format!("branch observation failed before movement: {denial:?}"),
                    ),
                )));
            }
        };
        if observed != *self.expected {
            return Err(Some(RelationalPublicationOutcome::stale(
                StaleRelationalBranchObservation::new(self.expected.clone(), observed),
            )));
        }

        // Capacity was reserved before head comparison so stale attempts still
        // release their exact preflight reservation. Only this coordinated,
        // unchanged head can turn capacity into a transfer witness.
        let head_retirement = match self
            .head_retirement
            .take()
            .expect("the owner issues one retirement reservation to one consuming cutover token")
            .bind_current_head()
        {
            Ok(reservation) => reservation,
            Err(denial) => {
                return Err(Some(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::RetentionOwner,
                        format!(
                            "selected head retention binding refused before movement: {denial:?}"
                        ),
                    ),
                )));
            }
        };

        if let (Some(effect), Some(binding)) = (self.companion.as_deref(), self.companion_binding)
            && let Err(stop) = effect.matches(binding)
        {
            return Err(Some(RelationalPublicationOutcome::deferred(
                RelationalPublicationDeferred::CompanionPreflight(stop),
            )));
        }
        let mut companion_cutover = match self.companion.as_deref_mut() {
            Some(effect) => match effect.enter_cutover() {
                Ok(cutover) => Some(cutover),
                Err(stop) => {
                    return Err(Some(RelationalPublicationOutcome::deferred(
                        RelationalPublicationDeferred::CompanionPreflight(stop),
                    )));
                }
            },
            None => None,
        };

        // Keep the caller's original request at the final pre-effect boundary;
        // a separate operation-control token cannot replace its first cause.
        if let Err(stop) = crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::PublicationCutover,
        ) {
            return Err(Some(RelationalPublicationOutcome::RequestStopped(stop)));
        }
        if allow_effect.as_mut().is_some_and(|allow| !allow()) {
            return Err(None);
        }

        let medium = self.medium;
        // A durable publisher waits for a publisher whose durable I/O holds
        // the reservation. Memory-only commits defer only behind other commits.
        let admission = match &medium {
            Some(_) => crate::runtime::state::subsystems::history::canonical_publication_routes::PatchPositionReservationAdmission::WaitWithin(execution),
            None => crate::runtime::state::subsystems::history::canonical_publication_routes::PatchPositionReservationAdmission::DeferBehindCommit(execution),
        };
        let symbol_meanings = symbol_meanings.take().expect(
            "the owner issues one prepared symbol publication to one consuming cutover token",
        );
        let (positioned_commit, head_retirement) = match route
            .record_performed_with_cutover(
                self.publication_cell.clone(),
                admission,
                deadline,
                allow_effect,
                medium,
                execution,
                |position| {
                    symbol_meanings
                        .publish()
                        .expect(/* Proof: PreparedSymbolPublication can only be issued by seal_for_publication after validating the canonical snapshot against the candidate interner. Its strong owner and retained ReservedSymbolMeaning tokens pin the unique identities allocated by reserve_identity; checked owner imports cannot replace either mapping, and recovery requires exclusive runtime access. */ "minted symbol reservations preserve owner meaning");
                    // The cutover is the only orchestration this port runs while the
                    // global patch-position reservation is held, so this is where a
                    // concurrent publisher can be made to meet it.
                    on_reservation();
                    let previous_root = publication_state.replace_with(self.next_state.take().expect(
                        "preflight prepares the one target state before the consuming cutover token",
                    ));
                    if let Some(companion) = &mut companion_cutover {
                        companion.install_at(position);
                    }
                    let transferred = head_retirement.transfer_head(next_root);
                    self.branch_head_versions
                        .move_head(Some(&observed_root), Some(next_root));
                    drop(previous_root);
                    drop(publication_state);
                    transferred
                },
            ) {
            Ok(positioned) => positioned,
            Err(crate::runtime::state::subsystems::history::canonical_publication_routes::CanonicalPublicationRecordError::CallerCutoffStopped) => {
                return Err(None);
            }
            Err(crate::runtime::state::subsystems::history::canonical_publication_routes::CanonicalPublicationRecordError::ReservationContended) => {
                return Err(Some(RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::PatchPositionReservationContended,
                )));
            }
            Err(crate::runtime::state::subsystems::history::canonical_publication_routes::CanonicalPublicationRecordError::ReservationWaitStopped(stop)) => {
                return Err(Some(RelationalPublicationOutcome::RequestStopped(stop)));
            }
            Err(crate::runtime::state::subsystems::history::canonical_publication_routes::CanonicalPublicationRecordError::CutoverDeadlineElapsed) => {
                return Err(Some(cutover_deadline_elapsed()));
            }
            Err(crate::runtime::state::subsystems::history::canonical_publication_routes::CanonicalPublicationRecordError::PositionCapacityExhausted) => {
                return Err(Some(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::PatchPositionCapacityExhausted,
                        "performed publication stream position capacity exhausted",
                    ),
                )));
            }
            Err(crate::runtime::state::subsystems::history::canonical_publication_routes::CanonicalPublicationRecordError::MemoryFrame(cause)) => {
                return Err(Some(memory_frame_stop_outcome(execution, cause)));
            }
            Err(crate::runtime::state::subsystems::history::canonical_publication_routes::CanonicalPublicationRecordError::Durable(stop)) => {
                return Err(Some(durable_stop_outcome(stop)));
            }
        };
        // Actual movement succeeded. Install only prepaid Native metadata;
        // keep owning payload cleanup parked until outside the caller's guard.
        settlement.install_prepared_indexes();
        drop(companion_cutover);
        drop(read_state);
        Ok((positioned_commit, head_retirement))
    }
}

/// The answer for a commit the medium did not make durable. Nothing was
/// installed, so a refusal is a pre-effect denial; an unknown fate is neither
/// denied nor performed.
fn durable_stop_outcome(
    stop: crate::durability::medium::RelationalDurableStepStop,
) -> RelationalPublicationOutcome {
    match stop {
        crate::durability::medium::RelationalDurableStepStop::Refused(refusal) => {
            RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::DurablePublicationRefused(refusal),
            )
        }
        crate::durability::medium::RelationalDurableStepStop::Unresolved(unresolved) => {
            RelationalPublicationOutcome::DurabilityUnresolved(unresolved)
        }
    }
}

fn memory_frame_stop_outcome(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    cause: crate::durability::canonical_commit_artifact::MemoryFramePreparationDenial,
) -> RelationalPublicationOutcome {
    use crate::durability::canonical_commit_artifact::MemoryFramePreparationDenial as Frame;
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
    let cause = match cause {
        Frame::Request(stop) => return RelationalPublicationOutcome::RequestStopped(stop),
        Frame::Artifact(cause) => cause,
    };
    if let Err(stop) = execution.consult()
        && stop
            != worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped,
            )
    {
        return RelationalPublicationOutcome::RequestStopped(stop);
    }
    if let Denial::Allocation(allocation) = &cause
        && let worth_execution::ExecutionAllocationDenialClass::Request(stop) =
            allocation.classify()
    {
        return RelationalPublicationOutcome::RequestStopped(stop);
    }
    RelationalPublicationOutcome::denied(RelationalPublicationDenial::CanonicalArtifactEncoding(
        cause,
    ))
}

pub(super) fn cutover_deadline_elapsed() -> RelationalPublicationOutcome {
    RelationalPublicationOutcome::denied(RelationalPublicationDenial::CutoverDeadlineElapsed)
}
