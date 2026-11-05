//! Continue the admitted original open without issuing identities or republishing it.
use super::*;
use crate::publication::{
    ActiveAttemptCustody, WorldHomeTransitionOpenResumeDenial as Denial,
    WorldHomeTransitionOpenResumeRefused as Refused, WorldOpenAttemptOriginal,
    WorldOpenedPublicationAttempt,
};
use crate::recovery::{RecoveredNativeCustody, WorldRecoveredUnpublishedEffects};
use worth_foundational::facade::{
    CompositeAttemptAssociation, HomeProgramTransitionPlanDescription,
    HomeProgramTransitionStepAssociation,
};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(in crate::lifecycle) fn resume_home_transition_open(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        plan: &HomeProgramTransitionPlanDescription,
        step: HomeProgramTransitionStepAssociation,
        original: &CompositeAttemptAssociation,
        admission: &worth_proof::AdmittedHomeProgramTransitionStep<
            worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
        >,
        claim: WorldRecoveredUnpublishedEffects,
        change: worth_relational::facade::mvcc::RelationalTransactionIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<WorldOpenedPublicationAttempt, Refused> {
        let resumed = (|| {
            let binding = self
                .state
                .durable
                .as_ref()
                .ok_or(Denial::NotHomeBound)?
                .clone();
            let commitment = plan
                .step_commitment(step, original)
                .map_err(|_| Denial::IdentityScope)?;
            let authority = binding
                .binding()
                .transition_authority()
                .ok_or(Denial::IdentityScope)?;
            // Scope/fence first, before inspecting any recovered claim payload.
            authority
                .admit_step(
                    admission,
                    &commitment,
                    &admission.caller_association_commitment(),
                )
                .map_err(Denial::Admission)?;
            execution.consult().map_err(|cause| {
                Denial::Durable(crate::durability::WorldDurableMediumDenial::Request(cause))
            })?;
            if !self.state.recovery.admits_recovered(&claim) {
                return Err(Denial::Recovery(
                    crate::recovery::RuntimeWorldRecoveryDenial::ForeignHandle,
                ));
            }
            let expected = claim.expected_head().clone();
            if claim.record.open.association != *original
                || claim.record.attempt.owner_identity() != self.owner_identity()
                || claim.record.attempt.owner_identity().get() != self.owner_identity().get()
                || !std::sync::Arc::ptr_eq(&claim.record.binding, &binding)
                || admission.source() != expected.basis().relational_basis().admission_identity()
            {
                return Err(Denial::IdentityScope);
            }
            let association = claim
                .caller_association_commitment(execution, plan, step)
                .map_err(Denial::Durable)?;
            authority
                .admit_step(admission, &commitment, &association)
                .map_err(Denial::Admission)?;
            match &*claim
                .record
                .native
                .lock()
                .unwrap_or_else(|cause| cause.into_inner())
            {
                RecoveredNativeCustody::OpenOnly => {}
                RecoveredNativeCustody::Relational(_) => {
                    return Err(Denial::NativeEffectAlreadyDurable)
                }
                RecoveredNativeCustody::Continuing(_) => {
                    return Err(Denial::NativeEffectAlreadyDurable)
                }
                RecoveredNativeCustody::Working => {
                    return Err(Denial::Recovery(
                        crate::recovery::RuntimeWorldRecoveryDenial::Busy,
                    ))
                }
                RecoveredNativeCustody::Released => return Err(Denial::ReleaseInProgress),
            }
            if claim
                .record
                .release
                .lock()
                .unwrap_or_else(|cause| cause.into_inner())
                .is_some()
                || claim
                    .record
                    .committed_release
                    .lock()
                    .unwrap_or_else(|cause| cause.into_inner())
                    .is_some()
            {
                return Err(Denial::ReleaseInProgress);
            }
            let current = self
                .admit_publication_source(&expected)
                .map_err(|cause| Denial::Source(cause.cause()))?;
            if let Some(cause) = self.reservation_denial(&expected, cancellation, deadline) {
                return Err(Denial::Source(cause.cause()));
            }
            if binding.occurrence_unresolved() {
                return Err(Denial::Durable(
                    crate::durability::WorldDurableMediumDenial::PublicationUnresolved,
                ));
            }
            let resolved = ResolvedExpectedProductHead::from_current(
                crate::publication::CompositeComponentIntent::relational_only(change),
                expected.clone(),
                &current,
            )
            .map_err(|_| Denial::IdentityScope)?;
            let association_bytes = crate::durability::encode_home_transition_association(
                execution, plan, step, original,
            )
            .map_err(Denial::Durable)?;
            let durable_custody = crate::durability::WorldDurableOpenCustody::readmitted(
                binding.clone(),
                claim.record.open.clone(),
                execution,
            )
            .map_err(Denial::Durable)?;
            let operation = self
                .reserve_operation_if_open_and_bootstrapped()
                .map_err(|()| Denial::Source(NoEffectCause::OwnerUnavailable))?;
            let slot = self
                .state
                .recovery
                .reserve_recovered_reactivation(&claim)
                .map_err(Denial::Recovery)?;
            let resources = reservation_steps::reserve_publication_resources(
                self,
                &expected,
                &claim.record.commit,
                Some(&claim.record.attempt),
                true,
                Some(slot),
            )
            .map_err(Denial::Source)?;
            let capacities = reservation_steps::resources_to_capacities(
                resources,
                self.state.clock.now(),
                claim.record.commit.clone(),
                claim.record.identity.clone(),
                operation,
            );
            let custody = ActiveAttemptCustody::reactivate_open(
                claim.record.attempt.clone(),
                expected,
                deadline,
                capacities,
                durable_custody.clone(),
            );
            Ok(WorldOpenedPublicationAttempt {
                original: WorldOpenAttemptOriginal {
                    binding,
                    subject: durable_custody.subject.clone(),
                    durable_custody,
                    association_bytes,
                    identity: claim.record.attempt.clone(),
                    resolved,
                    custody,
                    deadline,
                },
            })
        })();
        match resumed {
            Ok(opened) => Ok(opened),
            Err(cause) => Err(Refused {
                cause,
                custody: claim,
            }),
        }
    }
}
