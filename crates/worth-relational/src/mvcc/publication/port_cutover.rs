use std::sync::Arc;

use super::{
    PerformedRelationalCommit, PreparedBranchPublicationPreflight, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationFailure, RelationalPublicationFailureKind,
    RelationalPublicationOutcome, StaleRelationalBranchObservation,
};

impl PreparedBranchPublicationPreflight {
    pub(super) fn perform_cutover(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        deadline: Option<std::time::Instant>,
    ) -> RelationalPublicationOutcome {
        let _publication_lifecycle = self
            .movement
            .canonical_publication_route
            .enter_publication();
        if let Some(deadline) = deadline {
            if let Err(stop) = crate::authority::commit::pipeline::consult_stage(
                execution,
                crate::authority::commit::pipeline::CommitStage::PublicationCutover,
            ) {
                return RelationalPublicationOutcome::RequestStopped(stop);
            }
            if std::time::Instant::now() >= deadline.min(self.expires_at) {
                return narrowed_deadline_elapsed(
                    Some(deadline),
                    self.expires_at,
                    self.maximum_lifetime_millis,
                );
            }
        }
        if std::time::Instant::now() >= self.expires_at {
            return RelationalPublicationOutcome::deferred(
                RelationalPublicationDeferred::CandidateLifetimeExpired {
                    maximum_lifetime_millis: self.maximum_lifetime_millis,
                },
            );
        }
        match self
            .control
            .observe(crate::mvcc::RelationalInterruptionBoundary::BeforeCriticalSection)
        {
            Some(event)
                if event.interruption()
                    == crate::mvcc::RelationalOperationInterruption::Cancelled =>
            {
                self.next_basis
                    .inner
                    .retention_binding
                    .record_interruption(event);
                return RelationalPublicationOutcome::interrupted(event);
            }
            Some(event) => {
                self.next_basis
                    .inner
                    .retention_binding
                    .record_interruption(event);
                return RelationalPublicationOutcome::interrupted(event);
            }
            None => {}
        }
        let _critical_section =
            crate::mvcc::publication::current_read::PublicationReadCoordination::enter(
                &self.publication_cell,
                self.current_read.as_ref(),
            );
        if std::time::Instant::now() >= self.expires_at {
            return RelationalPublicationOutcome::deferred(
                RelationalPublicationDeferred::CandidateLifetimeExpired {
                    maximum_lifetime_millis: self.maximum_lifetime_millis,
                },
            );
        }
        if let Some(event) = self
            .control
            .observe(crate::mvcc::RelationalInterruptionBoundary::BeforeCriticalSection)
        {
            self.next_basis
                .inner
                .retention_binding
                .record_interruption(event);
            return RelationalPublicationOutcome::interrupted(event);
        }
        self.control.pause_inside_critical_section_for_test();
        let (mut publication_state, read_state) = match _critical_section.enter_states() {
            Ok(states) => states,
            Err(denial) => {
                return RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::CurrentReadDependency(denial),
                )
            }
        };
        match publication_state.lifecycle_posture() {
            crate::branch::RelationalBranchLifecyclePosture::Live => {}
            crate::branch::RelationalBranchLifecyclePosture::Archived => {
                return RelationalPublicationOutcome::denied(RelationalPublicationDenial::Archived);
            }
            crate::branch::RelationalBranchLifecyclePosture::Deleting => {
                return RelationalPublicationOutcome::denied(RelationalPublicationDenial::Deleting);
            }
        }
        let observed_cell = publication_state.snapshot_cell();
        let Some(observed_root) = observed_cell.root() else {
            return RelationalPublicationOutcome::failed(RelationalPublicationFailure::new(
                RelationalPublicationFailureKind::SelectedRootUnavailable,
                "selected branch root is unavailable before publication movement",
            ));
        };
        let observed = match crate::branch::descriptor_for_cell(&observed_cell, &observed_root) {
            Ok(descriptor) => descriptor,
            Err(denial) => {
                return RelationalPublicationOutcome::failed(RelationalPublicationFailure::new(
                    RelationalPublicationFailureKind::BranchObservation(denial.clone()),
                    format!("branch observation failed before movement: {denial:?}"),
                ));
            }
        };
        if observed != self.expected {
            return RelationalPublicationOutcome::stale(StaleRelationalBranchObservation::new(
                self.expected.clone(),
                observed,
            ));
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
                return RelationalPublicationOutcome::failed(RelationalPublicationFailure::new(
                    RelationalPublicationFailureKind::RetentionOwner,
                    format!("selected head retention binding refused before movement: {denial:?}"),
                ))
            }
        };

        if let (Some(effect), Some(binding)) = (&self.companion, &self.companion_binding)
            && let Err(stop) = effect.matches(binding)
        {
            return RelationalPublicationOutcome::deferred(
                RelationalPublicationDeferred::CompanionPreflight(stop),
            );
        }
        let mut companion_cutover = match self.companion.as_mut() {
            Some(effect) => match effect.enter_cutover() {
                Ok(cutover) => Some(cutover),
                Err(stop) => {
                    return RelationalPublicationOutcome::deferred(
                        RelationalPublicationDeferred::CompanionPreflight(stop),
                    )
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
            return RelationalPublicationOutcome::RequestStopped(stop);
        }

        let next_root = &self.movement.root;
        let reservation_pause = &self.control;
        let medium = self.medium.clone();
        // A durable publisher waits for a publisher whose durable I/O holds
        // the reservation. Memory-only commits defer only behind other commits.
        let admission = match &medium {
            Some(_) => crate::runtime::PatchPositionReservationAdmission::WaitWithin(execution),
            None => crate::runtime::PatchPositionReservationAdmission::DeferBehindCommit(execution),
        };
        let symbol_meanings = self.movement.symbol_meanings.take().expect(
            "the owner issues one prepared symbol publication to one consuming cutover token",
        );
        let (positioned_commit, head_retirement) = match self
            .movement
            .canonical_publication_route
            .record_performed_with_cutover(
                self.publication_cell.clone(),
                admission,
                Some(deadline.map_or(self.expires_at, |deadline| deadline.min(self.expires_at))),
                medium.as_deref(),
                execution,
                |position| {
                    symbol_meanings
                        .publish()
                        .expect(/* Proof: PreparedSymbolPublication can only be issued by seal_for_publication after validating the canonical snapshot against the candidate interner. Its strong owner and retained ReservedSymbolMeaning tokens pin the unique identities allocated by reserve_identity; checked owner imports cannot replace either mapping, and recovery requires exclusive runtime access. */ "minted symbol reservations preserve owner meaning");
                    // The cutover is the only orchestration this port runs while the
                    // global patch-position reservation is held, so this is where a
                    // concurrent publisher can be made to meet it.
                    reservation_pause.pause_holding_patch_position_reservation_for_test();
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
            Err(crate::runtime::CanonicalPublicationRecordError::ReservationContended) => {
                return RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::PatchPositionReservationContended,
                );
            }
            Err(crate::runtime::CanonicalPublicationRecordError::ReservationWaitStopped(stop)) => {
                return RelationalPublicationOutcome::RequestStopped(stop);
            }
            Err(crate::runtime::CanonicalPublicationRecordError::CutoverDeadlineElapsed) => {
                return narrowed_deadline_elapsed(deadline, self.expires_at, self.maximum_lifetime_millis);
            }
            Err(crate::runtime::CanonicalPublicationRecordError::PositionCapacityExhausted) => {
                return RelationalPublicationOutcome::failed(RelationalPublicationFailure::new(
                    RelationalPublicationFailureKind::PatchPositionCapacityExhausted,
                    "performed publication stream position capacity exhausted",
                ));
            }
            Err(crate::runtime::CanonicalPublicationRecordError::MemoryFrame(cause))=>{
                return memory_frame_stop_outcome(execution, cause);
            }
            Err(crate::runtime::CanonicalPublicationRecordError::Durable(stop)) => {
                return durable_stop_outcome(stop);
            }
        };
        // Actual movement succeeded. Install only prepaid Native metadata;
        // keep owning payload cleanup parked until outside the caller's guard.
        self.settlement_reservation
            .as_mut()
            .expect("the same cutover owns its reserved completion")
            .install_prepared_indexes();
        drop(companion_cutover);
        drop(read_state);
        let _critical_section = _critical_section.into_target_guard();
        self.control.pause_after_linearization_for_test();
        drop(_critical_section);
        let settlement_retention = head_retirement.replace_head_for_publication(
            self.candidate_retention
                .take()
                .expect("the consuming cutover owns candidate retention"),
            Arc::clone(&self.movement.root),
        );
        drop(_publication_lifecycle);
        self.movement
            .record_allocations
            .take()
            .expect("the consuming cutover owns prepared record allocations")
            .commit();
        let late_interruption = self
            .control
            .observe(crate::runtime::RelationalInterruptionBoundary::AfterLinearization);
        if let Some(interruption) = late_interruption {
            self.next_basis
                .inner
                .retention_binding
                .record_interruption(interruption);
        }
        // Authorizing the already-installed record is what disarms pre-effect
        // cleanup. From here the commit is recoverable by identity alone, and
        // the returned witness is only a witness.
        let record = self
            .settlement_reservation
            .take()
            .expect("one actual cutover authorizes its original settlement reservation")
            .authorize(
                Arc::clone(&positioned_commit),
                settlement_retention,
                late_interruption,
            );
        RelationalPublicationOutcome::performed(PerformedRelationalCommit::record(
            positioned_commit,
            self.next_basis.clone(),
            record,
            late_interruption,
        ))
    }
}

fn narrowed_deadline_elapsed(
    additional: Option<std::time::Instant>,
    candidate_expiry: std::time::Instant,
    maximum_lifetime_millis: u64,
) -> RelationalPublicationOutcome {
    if additional.is_some_and(|deadline| deadline <= candidate_expiry) {
        RelationalPublicationOutcome::denied(RelationalPublicationDenial::CutoverDeadlineElapsed)
    } else {
        RelationalPublicationOutcome::deferred(
            RelationalPublicationDeferred::CandidateLifetimeExpired {
                maximum_lifetime_millis,
            },
        )
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
    cause:crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial,
) -> RelationalPublicationOutcome {
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial as Denial;
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
