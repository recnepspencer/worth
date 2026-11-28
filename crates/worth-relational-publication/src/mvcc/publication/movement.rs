//! Compare the selected branch state and consume one prepared movement.
use worth_relational_contracts::internal_port::execution::monotonic_time::RelationalCutoverDeadline;
use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureOwnerPort as _;
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port as storage;

pub mod preflight;

use super::current_read::PublicationReadCoordination;
use super::CandidateCompanionBinding;
use super::PreparedPublicationCompanionEffect;
use super::RelationalPublicationOutcome;
use super::StaleRelationalBranchObservation;
use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailure;
use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureKind;
use contracts::mvcc::publication::outcome::refusals::RelationalPublicationDeferred;
use contracts::mvcc::publication::outcome::refusals::RelationalPublicationDenial;
use worth_relational_branch::internal_port::branch::reference::RelationalBranchReferenceCell;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::history::retention::head_retirement::TransferredHeadRetirement;

pub(super) type PerformedBranchMovement = (
    Arc<worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit>,
    TransferredHeadRetirement,
);

pub struct BranchMovementCutover<'a, 'cell> {
    pub coordination: &'a PublicationReadCoordination<'cell>,
    pub publication_cell:
        &'a worth_relational_branch::internal_port::branch::reference_publication_cell::RelationalBranchPublicationCell,
    pub expected: &'a worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor,
    pub next_state:
        &'a mut Option<worth_relational_branch::internal_port::branch::reference::RelationalBranchReferenceMutableState>,
    pub head_retirement:
        &'a mut Option<worth_relational_branch::internal_port::history::retention::head_retirement::RelationalHeadRetirementCapacity>,
    pub companion: Option<&'a mut PreparedPublicationCompanionEffect>,
    pub companion_binding: Option<&'a CandidateCompanionBinding>,
    pub medium:
        Option<&'a worth_relational_durability::internal_port::durability::published_medium::binding::RelationalDurableMediumBinding>,
    pub branch_head_versions:
        &'a worth_relational_branch::internal_port::runtime::state::subsystems::history::history_head_versions::BranchHeadVersionIndexAuthority,
}

pub struct PreparedCanonicalBranchMovement {
    pub record_allocations: Option<storage::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations>,
    pub symbol_meanings: Option<storage::runtime::state::subsystems::services::symbol_table::reservations::PreparedSymbolPublication>,
    next_cell: RelationalBranchReferenceCell,
    pub root: Arc<RelationalBranchRoot>,
    pub canonical_publication_route: crate::runtime::state::subsystems::history::canonical_publication_routes::PreparedCanonicalPublicationRoute,
}

impl PreparedCanonicalBranchMovement {
    pub(crate) fn new(
        record_allocations: storage::runtime::state::subsystems::record_identity::pending_allocations::PendingRecordAllocations,
        symbol_meanings: storage::runtime::state::subsystems::services::symbol_table::reservations::PreparedSymbolPublication,
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
    pub fn compare_and_move(
        &mut self,
        route: &crate::runtime::state::subsystems::history::canonical_publication_routes::PreparedCanonicalPublicationRoute,
        next_root: &Arc<RelationalBranchRoot>,
        symbol_meanings: &mut Option<
            storage::runtime::state::subsystems::services::symbol_table::reservations::PreparedSymbolPublication,
        >,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        deadline: Option<RelationalCutoverDeadline<'_>>,
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
            worth_relational_branch::internal_port::branch::lifecycle::RelationalBranchLifecyclePosture::Live => {}
            worth_relational_branch::internal_port::branch::lifecycle::RelationalBranchLifecyclePosture::Archived => {
                return Err(Some(RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::Archived,
                )));
            }
            worth_relational_branch::internal_port::branch::lifecycle::RelationalBranchLifecyclePosture::Deleting => {
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
        let observed = match worth_relational_branch::internal_port::branch::basis_admission::descriptor_for_cell(
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
    stop: storage::durability::medium::step_stop::RelationalDurableStepStop,
) -> RelationalPublicationOutcome {
    match stop {
        storage::durability::medium::step_stop::RelationalDurableStepStop::Refused(refusal) => {
            RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::DurablePublicationRefused(refusal),
            )
        }
        storage::durability::medium::step_stop::RelationalDurableStepStop::Unresolved(
            unresolved,
        ) => RelationalPublicationOutcome::DurabilityUnresolved(unresolved),
    }
}

fn memory_frame_stop_outcome(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    cause: worth_relational_durability::internal_port::durability::canonical_commit_artifact::memory_publication::denial::MemoryFramePreparationDenial,
) -> RelationalPublicationOutcome {
    use worth_relational_durability::internal_port::durability::canonical_commit_artifact::memory_publication::denial::MemoryFramePreparationDenial as Frame;
    use contracts::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
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

pub fn cutover_deadline_elapsed() -> RelationalPublicationOutcome {
    RelationalPublicationOutcome::denied(RelationalPublicationDenial::CutoverDeadlineElapsed)
}
