//! Complete actual metadata discovery precedes any recovered World stage.
use crate::durability::{
    decode_association, is_world_association, RecoveredCustodyDisposition, WorldDurableHomeBinding,
    WorldReadmissionDenial as Denial, WorldRecoveredJournalState,
};
use worth_relational::facade::durability::RelationalAssociatedEffectSettlement;

pub(super) fn readmit(
    native: &worth_relational::facade::runtime::RelationalRuntime,
    native_recovery: &worth_relational::facade::durability::RecoveredRelationalRuntimeAuthority,
    recovered: &WorldRecoveredJournalState,
    binding: &WorldDurableHomeBinding,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<super::native_effects::ReadmittedNativeEffects, Denial> {
    let subjects = native_recovery
        .discover_recovered_associated_effects(execution, native)
        .map_err(Denial::NativeDiscovery)?;
    let mut count = 0usize;
    for subject in &subjects {
        execution.consult().map_err(|cause| {
            Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
        })?;
        if subject
            .association()
            .is_some_and(|association| is_world_association(association.as_bytes()))
        {
            count = count.checked_add(1).ok_or(Denial::RecoveryCapacity)?;
        }
    }
    let mut effects = super::native_effects::NativeEffectAdmission::allocate(*execution, count)?;
    for subject in subjects {
        let Some(bytes) = subject
            .association()
            .map(|association| association.as_bytes())
        else {
            return Err(Denial::MalformedNativeAssociation);
        };
        if !is_world_association(bytes) {
            continue;
        }
        let association = decode_association(bytes, binding.owner().home())
            .map_err(|_| Denial::MalformedNativeAssociation)?;
        if association.attempt().owner() != binding.owner() {
            return Err(Denial::ForeignOwner);
        }
        let open = recovered
            .opens
            .get(&association.attempt())
            .ok_or(Denial::OrphanNativeAssociation)?;
        if association != open.association {
            return Err(Denial::ConflictingOpenAttempt);
        }
        match open.component_intent {
            crate::durability::WorldOpenComponentIntent::SignalOnly => {
                return Err(Denial::OccurrenceEvidenceMismatch)
            }
            crate::durability::WorldOpenComponentIntent::RelationalOnly
            | crate::durability::WorldOpenComponentIntent::RelationalAndSignal => {}
        }
        if effects.contains(&association.attempt()) {
            return Err(Denial::DuplicateNativeEffect);
        }
        let effect = native
            .durability_authority()
            .readmit_associated_effect(execution, &subject)
            .map_err(Denial::NativeDiscovery)?;
        if effect.settlement() == RelationalAssociatedEffectSettlement::PerformedPending {
            return Err(Denial::NativeSettlementRequired);
        }
        if let Some(RecoveredCustodyDisposition::Released(recorded)) = recovered
            .dispositions
            .get(&association.attempt())
            .map(|row| row.disposition())
            && recorded.as_ref().is_none_or(|recorded| {
                recorded.receipt() != subject.receipt()
                    || recorded.position() != subject.position().0
                    || recorded.digest() != subject.artifact_digest()
            })
        {
            return Err(Denial::DispositionEvidenceMismatch);
        }
        effects.push(&association.attempt(), effect, execution)?;
    }
    for (attempt, row) in &recovered.dispositions {
        let disposition = row.disposition();
        if let RecoveredCustodyDisposition::Released(recorded) = disposition
            && recorded.is_some() != effects.contains(attempt)
        {
            return Err(Denial::DispositionEvidenceMismatch);
        }
    }
    effects.seal()
}
