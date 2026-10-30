use std::{num::NonZeroU64, sync::atomic::Ordering, sync::Weak};

use worth_proof::TransitionOutcome;

use super::{
    PhysicalSubmissionState, PhysicalWorkSubmissionFailure, PhysicalWorkSubmissionOutcome,
    PhysicalWorkSubmissionReceipt, PhysicalWorkSubmissionStale,
};
use crate::physical_runtime::{
    work::{
        PhysicalMetadataReadWorkRequest, PhysicalMutationWorkRequest, PhysicalOperationIdentity,
        PhysicalReadWorkRequest, PhysicalWorkDurabilityRequirement, PhysicalWorkGeneration,
        PhysicalWorkIdentity, PhysicalWorkIntent, PhysicalWorkIntentParts,
        PhysicalWorkOperationFamily, PhysicalWorkRecoveryDisposition, PhysicalWorkSubmissionDenial,
    },
    LifecycleGeneration,
};

#[derive(Clone)]
pub struct PhysicalReadSubmission {
    pub(super) shared: Weak<PhysicalSubmissionState>,
    pub(super) generation: LifecycleGeneration,
}

#[derive(Clone)]
pub struct PhysicalMutationSubmission {
    pub(super) shared: Weak<PhysicalSubmissionState>,
    pub(super) generation: LifecycleGeneration,
}

impl PhysicalReadSubmission {
    pub fn submit_metadata(
        &self,
        request: PhysicalMetadataReadWorkRequest,
    ) -> PhysicalWorkSubmissionOutcome {
        submit(
            &self.shared,
            self.generation,
            PhysicalWorkIntentRequest {
                operation: PhysicalWorkOperationFamily::ArtifactMetadataRead,
                scope: request.scope,
                semantic_basis: request.semantic_basis,
                security: request.security,
                effect: crate::physical_runtime::work::PhysicalWorkEffectClass::ReadOnly,
                durability: PhysicalWorkDurabilityRequirement::ReadOnly,
                recovery: PhysicalWorkRecoveryDisposition::NoEffect,
            },
        )
    }

    pub fn submit(&self, request: PhysicalReadWorkRequest) -> PhysicalWorkSubmissionOutcome {
        submit(
            &self.shared,
            self.generation,
            PhysicalWorkIntentRequest {
                operation: PhysicalWorkOperationFamily::ArtifactRangeRead,
                scope: request.scope,
                semantic_basis: request.semantic_basis,
                security: request.security,
                effect: crate::physical_runtime::work::PhysicalWorkEffectClass::ReadOnly,
                durability: PhysicalWorkDurabilityRequirement::ReadOnly,
                recovery: PhysicalWorkRecoveryDisposition::NoEffect,
            },
        )
    }
}

impl PhysicalMutationSubmission {
    pub fn submit(&self, request: PhysicalMutationWorkRequest) -> PhysicalWorkSubmissionOutcome {
        submit(
            &self.shared,
            self.generation,
            PhysicalWorkIntentRequest {
                operation: request.operation,
                scope: request.scope,
                semantic_basis: request.semantic_basis,
                security: request.security,
                effect: request.effect,
                durability: request.durability,
                recovery: request.recovery,
            },
        )
    }
}

struct PhysicalWorkIntentRequest {
    operation: PhysicalWorkOperationFamily,
    scope: crate::physical_runtime::work::PhysicalWorkScope,
    semantic_basis: crate::physical_runtime::work::PhysicalWorkSemanticBasis,
    security: worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
    effect: crate::physical_runtime::work::PhysicalWorkEffectClass,
    durability: PhysicalWorkDurabilityRequirement,
    recovery: PhysicalWorkRecoveryDisposition,
}

fn submit(
    weak: &Weak<PhysicalSubmissionState>,
    generation: LifecycleGeneration,
    request: PhysicalWorkIntentRequest,
) -> PhysicalWorkSubmissionOutcome {
    let Some(shared) = weak.upgrade() else {
        return TransitionOutcome::stale(PhysicalWorkSubmissionStale::OwnerReleased).into();
    };
    let _activity = match shared.enter(generation) {
        Ok(activity) => activity,
        Err(stale) => return TransitionOutcome::stale(stale).into(),
    };
    if let Err(denial) = admit_submission_contracts(&shared, &request) {
        return TransitionOutcome::denied(denial).into();
    }
    let scope_members = request.scope.member_count();
    let semantic_bytes = request.semantic_basis.semantic_byte_width();
    let reservation = match shared.reserve(scope_members, semantic_bytes) {
        Ok(reservation) => reservation,
        Err(deferred) => return TransitionOutcome::deferred(deferred).into(),
    };
    let identity = match allocate_operation_identity(&shared) {
        Ok(identity) => identity,
        Err(failure) => return TransitionOutcome::failed(failure).into(),
    };
    let intent = match PhysicalWorkIntent::from_instance_owner(PhysicalWorkIntentParts {
        identity,
        operation: request.operation,
        scope: request.scope,
        semantic_basis: request.semantic_basis,
        security: request.security,
        effect: request.effect,
        durability: request.durability,
        signal_profile: shared.signal_profile,
        recovery: request.recovery,
    }) {
        Ok(intent) => intent,
        Err(denial) => {
            return TransitionOutcome::denied(PhysicalWorkSubmissionDenial::Declaration(denial))
                .into();
        }
    };
    let reservation = reservation.commit();
    shared.commands.push_declared(
        intent,
        reservation.scope_members,
        reservation.semantic_bytes,
    );
    shared.accounting.record_declared();
    TransitionOutcome::success(PhysicalWorkSubmissionReceipt {
        identity,
        signal_profile: shared.signal_profile,
    })
    .into()
}

fn admit_submission_contracts(
    shared: &PhysicalSubmissionState,
    request: &PhysicalWorkIntentRequest,
) -> Result<(), PhysicalWorkSubmissionDenial> {
    if !shared.bindings.admits(
        request.semantic_basis.aspect_identity(),
        request.semantic_basis.binding_stamp(),
    ) {
        return Err(PhysicalWorkSubmissionDenial::SemanticContractNotInstalled);
    }
    if !shared.bindings.admits_security(request.security) {
        return Err(PhysicalWorkSubmissionDenial::SecurityAuthorityMismatch);
    }
    Ok(())
}

pub(super) fn allocate_operation_identity(
    shared: &PhysicalSubmissionState,
) -> Result<PhysicalWorkIdentity, PhysicalWorkSubmissionFailure> {
    let sequence = {
        let update_counter = &shared.next_operation;
        let mut value = update_counter.load(Ordering::Relaxed);
        loop {
            let next_value = value.checked_add(1);
            let Some(updated_value) = next_value else {
                break Err(value);
            };
            match update_counter.compare_exchange_weak(
                value,
                updated_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => value = reloaded_value,
            }
        }
    }
    .map_err(|_| PhysicalWorkSubmissionFailure::OperationIdentityExhausted)?;
    let sequence = NonZeroU64::new(sequence)
        .expect("physical operation sequence starts at one and only increments");
    Ok(PhysicalWorkIdentity::from_instance_owner(
        shared.store,
        shared.runtime,
        PhysicalWorkGeneration::from_lifecycle(shared.generation),
        PhysicalOperationIdentity::from_owner_sequence(sequence),
    ))
}
