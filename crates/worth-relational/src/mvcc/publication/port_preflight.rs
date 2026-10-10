use std::sync::Arc;

use super::super::candidate::PreparedRelationalPublicationParts;
use super::{
    PreparedBranchPublicationPreflight, PreparedCanonicalBranchMovement,
    RelationalPublicationDeferred, RelationalPublicationDenial, RelationalPublicationFailure,
    RelationalPublicationFailureKind, RelationalPublicationOutcome,
};

impl PreparedCanonicalBranchMovement {
    pub(super) fn preflight(
        parts: PreparedRelationalPublicationParts,
        retention_binding: crate::history::retention::RelationalBranchRetentionBinding,
        branch_head_versions: crate::runtime::BranchHeadVersionIndexAuthority,
        companion_epoch: &super::super::CompanionRegistrationEpoch<'_>,
    ) -> Result<PreparedBranchPublicationPreflight, RelationalPublicationOutcome> {
        let PreparedRelationalPublicationParts {
            runtime_instance_id,
            candidate_id,
            publication_binding,
            expected,
            expected_root,
            publication_cell,
            movement,
            completion,
            candidate_retention,
            control,
            expires_at,
            maximum_lifetime_millis,
        } = parts;
        match control.observe(crate::mvcc::RelationalInterruptionBoundary::PublicationPreflight) {
            Some(event)
                if event.interruption()
                    == crate::mvcc::RelationalOperationInterruption::Cancelled =>
            {
                retention_binding.record_interruption(event);
                return Err(RelationalPublicationOutcome::interrupted(event));
            }
            Some(event) => {
                retention_binding.record_interruption(event);
                return Err(RelationalPublicationOutcome::interrupted(event));
            }
            None => {}
        }
        let next_state = movement.next_cell.state_snapshot();
        if publication_cell.runtime_instance_id() != expected.runtime_instance_id()
            || publication_cell.branch_id() != expected.branch_id()
        {
            return Err(RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::OwnerMismatch,
            ));
        }
        if movement.next_cell.identity().runtime_instance_id()
            != publication_cell.runtime_instance_id()
            || movement.next_cell.identity().branch_id() != publication_cell.branch_id()
            || movement.next_cell.root().as_ref().map(Arc::as_ptr)
                != Some(Arc::as_ptr(&movement.root))
        {
            return Err(RelationalPublicationOutcome::failed(
                RelationalPublicationFailure::new(
                    RelationalPublicationFailureKind::PreparedRootMismatch,
                    "prepared next root does not match its branch publication cell",
                ),
            ));
        }
        let next_descriptor =
            match crate::branch::descriptor_for_cell(&movement.next_cell, &movement.root) {
                Ok(descriptor) => descriptor,
                Err(denial) => {
                    return Err(RelationalPublicationOutcome::failed(
                        RelationalPublicationFailure::new(
                            RelationalPublicationFailureKind::PreparedBasisDescriptor(
                                denial.clone(),
                            ),
                            format!("prepared next basis failed before movement: {denial:?}"),
                        ),
                    ));
                }
            };
        let (companion, companion_binding) = match companion_epoch.active() {
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
            Ok(Some((generation, participant, budget))) => {
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
                    budget,
                );
                let effect = participant
                    .prepare(&mut context)
                    .map_err(|stop| companion_stop_outcome(stop, &publication_cell, &expected))?;
                effect
                    .matches(&binding)
                    .map_err(|stop| companion_stop_outcome(stop, &publication_cell, &expected))?;
                (Some(effect), Some(binding))
            }
        };
        let next_basis = crate::branch::issue_admitted_relational_branch_basis_with_retention(
            next_descriptor,
            movement.next_cell.identity().clone(),
            Arc::clone(&movement.root),
            publication_cell.clone(),
            &retention_binding,
        );
        let next_basis = match next_basis.and_then(|basis| publication_cell.register_basis(basis)) {
            Ok(basis) => basis,
            Err(crate::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted) => {
                return Err(RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::RetentionBackpressure,
                ));
            }
            Err(crate::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget) => {
                return Err(RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::OwnerUnavailable {
                        runtime_instance_id: expected.runtime_instance_id(),
                    },
                ));
            }
            Err(denial) => {
                return Err(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::NextBasisAdmission(denial.clone()),
                        format!("next basis admission failed before movement: {denial:?}"),
                    ),
                ));
            }
        };
        let head_retirement = match retention_binding.reserve_head_retirement(
            movement.next_cell.identity(),
            &expected_root,
            publication_cell.head_retention(),
        ) {
            Ok(reservation) => reservation,
            Err(
                crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted,
            ) => {
                return Err(RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::RetentionBackpressure,
                ));
            }
            Err(
                crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable,
            ) => {
                return Err(RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::OwnerUnavailable {
                        runtime_instance_id: expected.runtime_instance_id(),
                    },
                ));
            }
            Err(
                crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted,
            ) => {
                return Err(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::RetentionIdentityExhausted,
                        "head-retirement retention identity exhausted before movement",
                    ),
                ));
            }
            Err(denial) => {
                return Err(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::RetentionOwner,
                        format!("head-retirement reservation failed before movement: {denial:?}"),
                    ),
                ));
            }
        };
        // The pending settlement record is installed here, before the critical
        // section, so no observer can ever see a moved branch head whose owner
        // recovery lookup has no record. Every stop below this point releases
        // the reservation by dropping it.
        let published_snapshot_basis =
            crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_observation(
                &next_basis.observation(),
            );
        let settlement_reservation = match publication_binding.reserve_pending_settlement(
            movement.canonical_publication_route.commit_id(),
            runtime_instance_id,
            crate::runtime::ReservedRelationalSettlement {
                completion,
                published_snapshot_basis,
                control: control.clone(),
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
            companion,
            companion_binding,
            movement,
            expected,
            publication_cell,
            next_state,
            next_basis,
            settlement_reservation,
            head_retirement,
            candidate_retention,
            control,
            expires_at,
            maximum_lifetime_millis,
            branch_head_versions,
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
    if follows_concurrent_publication(&stop) {
        if let Some(observed) = observe_head(publication_cell) {
            if observed != *expected {
                return RelationalPublicationOutcome::stale(
                    super::StaleRelationalBranchObservation::new(expected.clone(), observed),
                );
            }
        }
    }
    map_companion_stop(stop)
}

const fn follows_concurrent_publication(stop: &super::super::CompanionPreflightStop) -> bool {
    use super::super::CompanionPreflightStop as Stop;
    match stop {
        Stop::TopologyPending | Stop::SelectedSourceMismatch => true,
        Stop::SelectedPositionUnavailable { .. }
        | Stop::ForeignCell
        | Stop::RegistrationChanged
        | Stop::WorkExhausted { .. }
        | Stop::WorkCounterOverflow
        | Stop::PreparationMemoryExhausted { .. }
        | Stop::PreparationMemoryCounterOverflow
        | Stop::RetainedCompanionCapacityExhausted { .. }
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
