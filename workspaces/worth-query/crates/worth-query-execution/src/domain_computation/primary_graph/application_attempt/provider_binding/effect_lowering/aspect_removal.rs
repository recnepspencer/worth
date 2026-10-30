//! Native readmission remains the owner of the actual whole-parent clear.
use super::*;
use worth_foundational::facade::{
    AbsenceLaw, AspectShape, PortableAspectContractBasis, PortableAspectPatchOperation,
    PortableRecordAspectPatch,
};
use worth_relational::facade::transactions::ApplyEntityAspectPatchIntent;
pub(super) fn lower(
    facts: &ObservedFactIndex<'_, impl FactSequence + ?Sized>,
    contracts: &AspectContracts<'_>,
    entity: &str,
    id: EntityId,
    basis: PortableAspectContractBasis,
    fields: Vec<AspectFieldLocator>,
) -> Result<WorthQueryLoweredProviderEffect, WorthQueryApplicationAttemptDenial> {
    let contract = &contracts(entity, basis.key())
        .ok_or_else(progression_denial)?
        .contract;
    if PortableAspectContractBasis::from_contract(contract) != basis
        || contract.absence() != AbsenceLaw::Optional
    {
        return Err(progression_denial());
    }
    let AspectShape::Struct(shape) = contract.shape() else {
        return Err(progression_denial());
    };
    if shape.fields().len() != fields.len() {
        return Err(progression_denial());
    }
    let mut steps = Vec::new();
    steps.try_reserve_exact(fields.len()).map_err(|_| WorthQueryApplicationAttemptDenial::new(
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::AllocationDenied,
        "whole-aspect effect steps"))?;
    let mut absent = None;
    for (field, locator) in shape.fields().iter().zip(&fields) {
        if locator.aspect().aspect_key() != contract.key()
            || locator.field_path().fields() != [field.key().clone()]
        {
            return Err(progression_denial());
        }
        let now_absent = facts.aspect_absent(id, locator)?;
        if absent.is_some_and(|before| before != now_absent) {
            return Err(progression_denial());
        }
        absent = Some(now_absent);
        steps.push(effect_step(WorthQueryProvisionalEffectAction::Retire {
            target_identity: facts.field_identity(id, locator)?.into(),
        })?);
    }
    if absent == Some(true) {
        return Ok(WorthQueryLoweredProviderEffect::ObservedNoop { steps });
    }
    mutation(
        steps,
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
            ApplyEntityAspectPatchIntent {
                entity_id: id,
                aspect_patch: PortableRecordAspectPatch::new(vec![
                    PortableAspectPatchOperation::ClearWhole { basis },
                ]),
            },
        )),
    )
}
