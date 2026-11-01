//! Original preflight lends a single cutover; all owning cleanup stays outside it.
use super::*;
use worth_execution::ExecutionRequest;

pub(super) enum CutoverProfile {
    Ordinary,
    MemoryOnly,
}

/// A one-use, borrowed memory-only cutover. The original request, owner and
/// configuration epochs stay alive in its issuing receiver. No companion
/// preparation, durable medium callback or payload destruction runs here.
pub struct PreparedMemoryRelationalCutover<'cutover, 'policy, 'request> {
    inner: PreparedRelationalCutover<'cutover, 'policy, 'request>,
}

impl PreparedMemoryRelationalCutover<'_, '_, '_> {
    /// Perform once. Actual Native custody is returned by the outer receiver
    /// after the handoff ends, so it cannot be settled or destroyed in a guard.
    pub fn perform(self) {
        self.inner.perform();
    }
}

pub(super) struct PreparedRelationalCutover<'cutover, 'policy, 'request> {
    prepared: &'cutover mut PreparedBranchPublicationPreflight,
    execution: &'cutover ExecutionRequest<'policy, 'request>,
    outcome: &'cutover mut Option<RelationalPublicationOutcome>,
}

impl PreparedRelationalCutover<'_, '_, '_> {
    pub(super) fn perform(self) {
        *self.outcome = Some(self.prepared.perform_cutover(self.execution));
    }
}

/// The caller's handoff result is distinct from the original Native effect.
/// None means the token was unused and Native performed nothing.
pub struct ScopedRelationalPublication<R> {
    handoff: R,
    publication: Option<RelationalPublicationOutcome>,
}

impl<R> ScopedRelationalPublication<R> {
    pub fn into_parts(self) -> (R, Option<RelationalPublicationOutcome>) {
        (self.handoff, self.publication)
    }
}

impl RelationalPublicationPort {
    /// Finish original companion preparation before lending one memory-only
    /// cutover. Destroy prepared/retired companion payloads after the callback,
    /// including refusal, an unused token and unwinding. No raw plan escapes.
    pub fn with_prepared_memory_cutover<'policy, 'request, R>(
        &self,
        execution: &ExecutionRequest<'policy, 'request>,
        candidate: PreparedRelationalCommitCandidate,
        handoff: impl for<'cutover> FnOnce(
            PreparedMemoryRelationalCutover<'cutover, 'policy, 'request>,
        ) -> R,
    ) -> Result<ScopedRelationalPublication<R>, RelationalPublicationOutcome> {
        let mut handoff = Some(handoff);
        let mut result = None;
        let publication = self.with_prepared_publication(
            execution,
            candidate,
            None,
            CutoverProfile::MemoryOnly,
            &mut |inner| {
                result = Some(handoff
                    .take()
                    .expect("the original preflight lends one token")(
                    PreparedMemoryRelationalCutover { inner },
                ));
            },
        )?;
        Ok(ScopedRelationalPublication {
            handoff: result.expect("successful preflight invokes the handoff once"),
            publication,
        })
    }

    // Concrete core: caller-specific result packaging stays in the small shell.
    pub(super) fn with_prepared_publication<'policy, 'request>(
        &self,
        execution: &ExecutionRequest<'policy, 'request>,
        candidate: PreparedRelationalCommitCandidate,
        admission: Option<crate::durability::AdmittedRecoveredCodecPublication>,
        profile: CutoverProfile,
        handoff: &mut dyn for<'cutover> FnMut(
            PreparedRelationalCutover<'cutover, 'policy, 'request>,
        ),
    ) -> Result<Option<RelationalPublicationOutcome>, RelationalPublicationOutcome> {
        if candidate.runtime_instance_id() != self.runtime_instance_id
            || !candidate.belongs_to_publication_owner(&self.publication_binding)
        {
            return Err(RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id,
                    actual_runtime_instance_id: candidate.runtime_instance_id(),
                },
            ));
        }
        if let Err(stop) = crate::authority::commit::pipeline::consult_stage(
            execution,
            crate::authority::commit::pipeline::CommitStage::Publication,
        ) {
            self.history.record_candidate_discard(candidate.branch());
            return Err(RelationalPublicationOutcome::RequestStopped(stop));
        }
        self.history.record_publication_attempt(candidate.branch());
        let branch = candidate.branch().clone();
        let result =
            self.with_prepared_consulted(execution, candidate, admission, profile, handoff);
        if !matches!(
            &result,
            Ok(Some(RelationalPublicationOutcome::Performed(_)))
        ) {
            self.history.record_candidate_discard(&branch);
        }
        result
    }

    fn with_prepared_consulted<'policy, 'request>(
        &self,
        execution: &ExecutionRequest<'policy, 'request>,
        candidate: PreparedRelationalCommitCandidate,
        admission: Option<crate::durability::AdmittedRecoveredCodecPublication>,
        profile: CutoverProfile,
        handoff: &mut dyn for<'cutover> FnMut(
            PreparedRelationalCutover<'cutover, 'policy, 'request>,
        ),
    ) -> Result<Option<RelationalPublicationOutcome>, RelationalPublicationOutcome> {
        if matches!(profile, CutoverProfile::MemoryOnly) && self.medium.is_some() {
            return Err(RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::ScopedCutoverDurabilityUnsupported,
            ));
        }
        if self.medium.is_some() && candidate.requires_retained_selection_profile() {
            return Err(RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::RetainedEntitySelectionDurabilityUnsupported,
            ));
        }
        let custody = candidate.recovered_codec_publication_custody();
        if !match (&custody, &admission) {
            (None, None) => true,
            (Some(custody), Some(admission)) => admission.admits(custody),
            _ => false,
        } {
            return Err(RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::RecoveredCodecTransitionRequired,
            ));
        }
        let Some(_owner_operation) = self.owner_binding.admit() else {
            return Err(RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::OwnerUnavailable {
                    runtime_instance_id: self.runtime_instance_id,
                },
            ));
        };
        let epoch = self.configuration.operation();
        let expected_generation = epoch.schema_contract_runtime.custom_invariant_generation;
        if candidate.custom_invariant_generation() != expected_generation {
            return Err(RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::StaleInvariantGeneration {
                    expected_generation,
                    actual_generation: candidate.custom_invariant_generation(),
                },
            ));
        }
        if candidate.lifetime_expired() {
            return Err(RelationalPublicationOutcome::deferred(
                RelationalPublicationDeferred::CandidateLifetimeExpired {
                    maximum_lifetime_millis: candidate.maximum_lifetime_millis,
                },
            ));
        }
        // This original epoch spans both user preparation and the fixed swap.
        let companion_epoch = self.publication_binding.companion_epoch().map_err(|_| {
            RelationalPublicationOutcome::deferred(
                RelationalPublicationDeferred::CompanionRegistrationPending,
            )
        })?;
        match companion_epoch.active() {
            Ok(_) => {}
            Err(super::super::PublicationCompanionRegistrationStop::RebindRequired) => {
                return Err(RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::CompanionRebindRequired,
                ))
            }
            Err(_) => {
                return Err(RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::CompanionRegistrationPending,
                ))
            }
        }
        let parts = candidate
            .into_publication_parts()
            .map_err(RelationalPublicationOutcome::deferred)?;
        let retention_binding =
            parts
                .publication_cell
                .head_retention()
                .binding()
                .map_err(|_| {
                    RelationalPublicationOutcome::denied(
                        RelationalPublicationDenial::OwnerUnavailable {
                            runtime_instance_id: self.runtime_instance_id,
                        },
                    )
                })?;
        let mut prepared = PreparedCanonicalBranchMovement::preflight(
            execution,
            parts,
            retention_binding,
            self.branch_head_versions.clone(),
            &companion_epoch,
            self.medium.clone(),
        )?;
        let mut outcome = None;
        handoff(PreparedRelationalCutover {
            prepared: &mut prepared,
            execution,
            outcome: &mut outcome,
        });
        // Any owning companion payload, including retired T, dies here after
        // the caller's guard ends. The actual outcome remains in this slot.
        drop(prepared);
        Ok(outcome)
    }
}
