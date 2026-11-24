use super::super::candidate::PreparedRelationalPublicationParts;
use super::{
    PreparedBranchPublicationPreflight, RelationalPublicationDeferred, RelationalPublicationDenial,
    RelationalPublicationFailure, RelationalPublicationFailureKind, RelationalPublicationOutcome,
};
use worth_relational_contracts::internal_port::mvcc::publication::outcome::failure::RelationalPublicationFailureOwnerPort as _;

impl PreparedBranchPublicationPreflight {
    pub(super) fn preflight(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        parts: PreparedRelationalPublicationParts,
        retention_binding: crate::history::retention::RelationalBranchRetentionBinding,
        branch_head_versions: crate::runtime::BranchHeadVersionIndexAuthority,
        companion_epoch: &super::super::CompanionRegistrationEpoch<'_>,
        medium: Option<
            std::sync::Arc<crate::durability::published_medium::RelationalDurableMediumBinding>,
        >,
    ) -> Result<PreparedBranchPublicationPreflight, RelationalPublicationOutcome> {
        let PreparedRelationalPublicationParts {
            current_read,
            companion_read,
            managed_source_reads,
            runtime_instance_id,
            candidate_id,
            publication_binding,
            expected,
            expected_root,
            publication_cell,
            movement,
            mut completion,
            candidate_retention,
            control,
        } = parts;
        if let Some(event) =
            control.observe(crate::runtime::RelationalInterruptionBoundary::PublicationPreflight)
        {
            retention_binding.record_interruption(event);
            return Err(RelationalPublicationOutcome::interrupted(event));
        }
        let (next_state, next_descriptor) =
            movement.select_next_basis(&publication_cell, &expected)?;
        let (companion, companion_binding) = match companion_epoch.active() {
            Ok(None) if companion_read.is_some() || managed_source_reads.is_some() => {
                return Err(companion_stop_outcome(
                    super::super::CompanionPreflightStop::RegistrationChanged,
                    &publication_cell,
                    &expected,
                ))
            }
            Ok(None) => (None, None),
            Err(super::super::PublicationCompanionRegistrationStop::RebindRequired) => {
                return Err(RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::CompanionRebindRequired,
                ));
            }
            Err(_) => {
                return Err(RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::CompanionRegistrationPending,
                ))
            }
            Ok(Some((generation, participant))) => {
                let envelope = movement.root.canonical_envelope().ok_or_else(|| {
                    RelationalPublicationOutcome::failed(RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::PreparedRootMismatch,
                        "prepared companion root has no canonical envelope",
                    ))
                })?;
                let expected_position = match expected_root.commit_id() {
                    Some(commit_id) => Some(
                        movement
                            .canonical_publication_route
                            .selected_position(commit_id)
                            .ok_or_else(|| {
                                map_companion_stop(
                                    super::super::CompanionPreflightStop::SelectedPositionUnavailable {
                                        commit_id,
                                    },
                                )
                            })?,
                    ),
                    None => None,
                };
                let binding = super::super::CandidateCompanionBinding {
                    runtime_instance_id,
                    registration_generation: generation,
                    candidate_id,
                    branch_id: expected.branch_id().clone(),
                    expected_root_id: expected_root.id(),
                    expected_commit_id: expected_root.commit_id(),
                    expected_position,
                    next_root_id: movement.root.id(),
                    next_commit_id: envelope.commit.commit_id,
                };
                let mut context = super::super::PublicationCompanionPreflight::new(
                    binding.clone(),
                    envelope,
                    &control,
                    &completion.source_interpretation,
                    execution,
                )
                .map_err(|stop| companion_stop_outcome(stop, &publication_cell, &expected))?;
                let prepared = participant.prepare(&mut context);
                // A companion cannot hide an original request stop behind
                // either a successful preparation or its own refusal.
                crate::authority::commit::pipeline::consult_stage(
                    execution,
                    crate::authority::commit::pipeline::CommitStage::Publication,
                )
                .map_err(RelationalPublicationOutcome::RequestStopped)?;
                let effect = prepared
                    .map_err(|stop| companion_stop_outcome(stop, &publication_cell, &expected))?;
                effect
                    .matches(&binding)
                    .map_err(|stop| companion_stop_outcome(stop, &publication_cell, &expected))?;
                if let Some(condition) = &companion_read {
                    effect.matches_read_condition(condition).map_err(|stop| {
                        companion_stop_outcome(stop, &publication_cell, &expected)
                    })?;
                }
                if let Some(condition) = &managed_source_reads {
                    effect
                        .matches_managed_source_reads(condition)
                        .map_err(|stop| {
                            companion_stop_outcome(stop, &publication_cell, &expected)
                        })?;
                }
                effect
                    .attach_managed_source_root(&movement.root)
                    .map_err(|stop| companion_stop_outcome(stop, &publication_cell, &expected))?;
                (Some(effect), Some(binding))
            }
        };
        let (next_basis, head_retirement) = movement.retain_for_cutover(
            next_descriptor,
            &retention_binding,
            &publication_cell,
            &expected_root,
        )?;
        // The pending settlement record is installed here, before the critical
        // section, so no observer can ever see a moved branch head whose owner
        // recovery lookup has no record. Every stop below this point releases
        // the reservation by dropping it.
        let published_snapshot_basis =
            crate::history::retention::VisibilitySnapshotBasis::from_observation(
                &next_basis.observation(),
            );
        completion.admit_prepared_indexes(execution)?;
        let settlement_reservation = match publication_binding.reserve_pending_settlement(
            movement.canonical_publication_route.commit_id(),
            runtime_instance_id,
            crate::runtime::ReservedRelationalSettlement {
                completion,
                published_snapshot_basis,
                control: control.clone(),
                envelope_cell: movement.canonical_publication_route.envelope_residency(),
            },
        ) {
            Ok(reservation) => reservation,
            Err(crate::runtime::RelationalSettlementReservationDenial::OwnerUnavailable) => {
                return Err(RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::OwnerUnavailable {
                        runtime_instance_id: expected.runtime_instance_id(),
                    },
                ));
            }
            Err(crate::runtime::RelationalSettlementReservationDenial::DuplicateCommitIdentity) => {
                return Err(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::PendingSettlementIdentityConflict,
                        "another pending settlement already owns this commit identity",
                    ),
                ));
            }
        };
        Ok(PreparedBranchPublicationPreflight {
            current_read,
            companion,
            companion_binding,
            movement,
            expected,
            publication_cell,
            next_state: Some(next_state),
            next_basis,
            settlement_reservation: Some(settlement_reservation),
            head_retirement: Some(head_retirement),
            candidate_retention: Some(candidate_retention),
            control,
            branch_head_versions,
            medium,
        })
    }
}

fn map_companion_stop(stop: super::super::CompanionPreflightStop) -> RelationalPublicationOutcome {
    match stop {
        super::super::CompanionPreflightStop::Interrupted(event) => {
            RelationalPublicationOutcome::interrupted(event)
        }
        other => RelationalPublicationOutcome::deferred(
            RelationalPublicationDeferred::CompanionPreflight(other),
        ),
    }
}

/// A stale expected head wins over a companion deferral. A concurrent
/// publisher on the same head makes the companion report a held reservation or
/// a moved root; the branch head is re-observed (briefly, outside the
/// publication critical section, never waiting on the reservation) and a
/// moved head is reported as the stale observation the cutover would report.
/// Only an unchanged head leaves the stop a deferral to retry unchanged.
fn companion_stop_outcome(
    stop: super::super::CompanionPreflightStop,
    publication_cell: &crate::branch::RelationalBranchPublicationCell,
    expected: &crate::branch::RelationalBranchBasisDescriptor,
) -> RelationalPublicationOutcome {
    if follows_concurrent_publication(&stop)
        && let Some(observed) = observe_head(publication_cell)
        && observed != *expected
    {
        return RelationalPublicationOutcome::stale(super::StaleRelationalBranchObservation::new(
            expected.clone(),
            observed,
        ));
    }
    map_companion_stop(stop)
}

const fn follows_concurrent_publication(stop: &super::super::CompanionPreflightStop) -> bool {
    use super::super::CompanionPreflightStop as Stop;
    match stop {
        Stop::TopologyPending | Stop::SelectedSourceMismatch => true,
        Stop::Execution(_)
        | Stop::SelectedPositionUnavailable { .. }
        | Stop::ForeignCell
        | Stop::RegistrationChanged
        | Stop::ManagedSourceNotPrepared
        | Stop::ManagedSourceChanged
        | Stop::WorkCounterOverflow
        | Stop::PreparationMemoryCounterOverflow
        | Stop::Interrupted(_) => false,
    }
}

/// The same head observation the cutover makes. A branch without a selected
/// root or an unreadable descriptor keeps the original stop; the cutover owns
/// those failures.
fn observe_head(
    publication_cell: &crate::branch::RelationalBranchPublicationCell,
) -> Option<crate::branch::RelationalBranchBasisDescriptor> {
    let observed_cell = publication_cell.enter_state().snapshot_cell();
    let observed_root = observed_cell.root()?;
    crate::branch::descriptor_for_cell(&observed_cell, &observed_root).ok()
}
