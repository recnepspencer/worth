//! Complete actual metadata discovery precedes any recovered World stage.
use crate::durability::{
    decode_association, is_world_association, RecoveredCustodyDisposition, WorldDurableHomeBinding,
    WorldReadmissionDenial as Denial, WorldRecoveredJournalState, WorldRelationalReleaseEvidence,
};
use std::collections::BTreeMap;
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity;
use worth_relational::facade::durability::{
    ReadmittedRelationalAssociatedEffect, RelationalAssociatedEffectSettlement,
};

pub(super) fn readmit(
    native: &worth_relational::facade::runtime::RelationalRuntime,
    recovered: &WorldRecoveredJournalState,
    binding: &WorldDurableHomeBinding,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<
    BTreeMap<DurableCompositePublicationAttemptIdentity, ReadmittedRelationalAssociatedEffect>,
    Denial,
> {
    let mut effects = BTreeMap::new();
    for subject in native
        .durability_authority()
        .discover_all_associated_effects(execution)
        .map_err(Denial::NativeDiscovery)?
    {
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
        if effects.contains_key(&association.attempt()) {
            return Err(Denial::DuplicateNativeEffect);
        }
        let effect = native
            .durability_authority()
            .readmit_associated_effect(execution, &subject)
            .map_err(Denial::NativeDiscovery)?;
        if effect.settlement() == RelationalAssociatedEffectSettlement::PerformedPending {
            return Err(Denial::NativeSettlementRequired);
        }
        if let Some(RecoveredCustodyDisposition::Released(recorded)) =
            recovered.dispositions.get(&association.attempt())
        {
            let actual = WorldRelationalReleaseEvidence {
                receipt: subject.receipt().clone(),
                position: subject.position().0,
                digest: *subject.artifact_digest(),
            };
            if recorded.as_ref() != Some(&actual) {
                return Err(Denial::DispositionEvidenceMismatch);
            }
        }
        effects.insert(association.attempt(), effect);
    }
    for (attempt, disposition) in &recovered.dispositions {
        if let RecoveredCustodyDisposition::Released(recorded) = disposition
            && recorded.is_some() != effects.contains_key(attempt)
        {
            return Err(Denial::DispositionEvidenceMismatch);
        }
    }
    Ok(effects)
}
