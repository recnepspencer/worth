//! The current receiver continues only its authenticated original durable effect.
use super::*;
use crate::publication::{
    WorldHomeTransitionEffectDenial as Denial, WorldHomeTransitionEffectOutcome as Outcome,
};
use crate::recovery::{
    OriginalContinuationAdvance, RecoveredNativeContinuationLoan, RecoveredNativeCustody,
    WorldRecoveredUnpublishedEffects,
};
use worth_foundational::facade::{
    CompositeAttemptAssociation, HomeProgramTransitionPlanDescription,
    HomeProgramTransitionStepAssociation,
};
mod control;
mod preparation;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(in crate::lifecycle) fn continue_home_transition_effect(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        plan: &HomeProgramTransitionPlanDescription,
        step: HomeProgramTransitionStepAssociation,
        original: &CompositeAttemptAssociation,
        admission: &worth_proof::AdmittedHomeProgramTransitionStep<
            worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
        >,
        claim: WorldRecoveredUnpublishedEffects,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Outcome {
        let result = (|| {
            let binding = self.state.durable.as_ref().ok_or(Denial::IdentityScope)?;
            let authority = binding
                .binding()
                .transition_authority()
                .ok_or(Denial::IdentityScope)?;
            let commitment = plan.step_commitment(step, original).map_err(Denial::Plan)?;
            // No claim payload, live read or provider runs before the attached seal/fence.
            authority
                .admit_step(
                    admission,
                    &commitment,
                    &admission.caller_association_commitment(),
                )
                .map_err(Denial::Admission)?;
            if !self.state.recovery.admits_recovered(&claim) {
                return Err(Denial::Recovery(
                    crate::recovery::RuntimeWorldRecoveryDenial::ForeignHandle,
                ));
            }
            let record = &claim.record;
            if !std::sync::Arc::ptr_eq(&record.binding, binding)
                || record.open.association != *original
                || record.attempt.owner_identity() != self.owner_identity()
                || admission.source()
                    != record
                        .expected
                        .basis()
                        .relational_basis()
                        .admission_identity()
                || plan
                    .predecessor()
                    .generation()
                    .get()
                    .checked_add(u64::from(step.step()))
                    != Some(record.expected.reference_generation().get())
            {
                return Err(Denial::IdentityScope);
            }
            request.consult().map_err(|cause| {
                Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
            })?;
            if let Some(cause) = self.reservation_denial(&record.expected, cancellation, deadline) {
                return Err(Denial::Source(cause.cause()));
            }
            let association = admission.caller_association_commitment();
            if record
                .release
                .lock()
                .unwrap_or_else(|cause| cause.into_inner())
                .is_some()
                || record
                    .committed_release
                    .lock()
                    .unwrap_or_else(|cause| cause.into_inner())
                    .is_some()
            {
                return Err(Denial::ReleaseInProgress);
            }
            let mut loan =
                RecoveredNativeContinuationLoan::take(record).map_err(Denial::Recovery)?;
            match loan.state() {
                RecoveredNativeCustody::OpenOnly => return Err(Denial::NativeEffectRequired),
                RecoveredNativeCustody::Relational(native) => {
                    let expected = crate::durability::encode_home_transition_association(
                        request, plan, step, original,
                    )
                    .map_err(Denial::Medium)?;
                    if native
                        .custody()
                        .effect()
                        .subject()
                        .association()
                        .map(|value| value.as_bytes())
                        != Some(expected.elements())
                    {
                        return Err(Denial::NativeEffectMismatch);
                    }
                    use sha2::Digest;
                    let actual: [u8; 32] = sha2::Sha256::digest(expected.elements()).into();
                    authority
                        .admit_step(admission, &commitment, &actual)
                        .map_err(Denial::Admission)?;
                    let prepared = self.prepare_original_home_continuation(
                        request,
                        plan,
                        step,
                        association,
                        record,
                        native.custody(),
                    )?;
                    loan.start(prepared);
                }
                RecoveredNativeCustody::Continuing(work) => {
                    if !work.admits(plan.commitment(), step, association) {
                        return Err(Denial::IdentityScope);
                    }
                }
                RecoveredNativeCustody::Working => {
                    return Err(Denial::Recovery(
                        crate::recovery::RuntimeWorldRecoveryDenial::Busy,
                    ))
                }
                RecoveredNativeCustody::Released => return Err(Denial::ReleaseInProgress),
            }
            let RecoveredNativeCustody::Continuing(work) = loan.state() else {
                unreachable!("the same original is installed before advancing")
            };
            let outcome = work.advance(request)?;
            if matches!(&outcome, OriginalContinuationAdvance::Completed(_)) {
                *loan.state() = RecoveredNativeCustody::Released;
                self.state.recovery.remove_recovered(record);
            }
            Ok(outcome)
        })();
        match result {
            Ok(OriginalContinuationAdvance::Completed(publication)) => {
                Outcome::Continued(publication)
            }
            Ok(OriginalContinuationAdvance::Pending) => Outcome::Pending(claim),
            Ok(OriginalContinuationAdvance::EvidenceRequired) => Outcome::EvidenceRequired(claim),
            Ok(OriginalContinuationAdvance::Indeterminate) => Outcome::Indeterminate(claim),
            Ok(OriginalContinuationAdvance::ReopenRequired) => Outcome::ReopenRequired(claim),
            Err(cause) => Outcome::Refused {
                cause,
                custody: claim,
            },
        }
    }
}
