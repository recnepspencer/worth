//! Aggregate effects consume only the complete groups validated by the parent court.
use super::*;

pub(super) fn validate(
    envelope: &CanonicalCommitEnvelope,
    groups: &[Arc<CanonicalRequiredOutputOrigin>],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), Stop> {
    if !envelope.record_allocations.is_empty() {
        return Err("codec disposition contains unrelated native allocations".into());
    }
    let event = &envelope.required_output_codec_retirements[0];
    // Every non-program mutation belongs to an explicitly recorded complete group.
    let mut activation_updates = 0usize;
    for intent in &envelope.merged_plan.merged_intents {
        charge(
            context,
            envelope
                .required_output_codec_retirements
                .len()
                .saturating_add(1),
        )?;
        match intent {
            MutationIntent::Entity(EntityMutationIntent::Delete(delete)) => {
                if !covered_entity(groups, delete.entity_id, context)? {
                    return Err("codec disposition deletes an unowned or incomplete member".into());
                }
            }
            MutationIntent::Relation(RelationMutationIntent::Delete(delete)) => {
                if !covered_relation(groups, delete.relation_id, context)? {
                    return Err("codec disposition deletes an unowned incident relation".into());
                }
            }
            MutationIntent::Entity(EntityMutationIntent::UpdateFields(update))
                if update.entity_id == event.activation.entity_id =>
            {
                activation_updates += 1;
                if update.fields
                    != crate::transactions::data::AspectFieldPatch::from(
                        std::collections::BTreeMap::from([(
                            event.activation.field_locator.clone(),
                            event.activation.replacement.clone(),
                        )]),
                    )
                {
                    return Err("codec disposition changes protected program fields".into());
                }
            }
            // Read demands carry no write or retirement authority. Every actual
            // canonical patch is independently covered below.
            MutationIntent::Entity(EntityMutationIntent::Revalidate(_))
            | MutationIntent::Relation(RelationMutationIntent::Revalidate(_)) => {}
            _ => return Err("codec disposition has an unrelated native mutation".into()),
        }
    }
    if activation_updates != 1 {
        return Err("codec disposition lacks one exact protected program update".into());
    }
    for (index, patch) in envelope
        .patch
        .authoritative_record_patches
        .iter()
        .enumerate()
    {
        charge(context, index.saturating_add(1))?;
        if envelope.patch.authoritative_record_patches[..index]
            .iter()
            .any(|prior| prior.target == patch.target)
        {
            return Err("codec disposition duplicates a canonical record patch".into());
        }
        let covered = match patch.target {
            RecordRef::Entity(id) if id == event.activation.entity_id => true,
            RecordRef::Entity(id) => covered_entity(groups, id, context)?,
            RecordRef::Relation(id) => covered_relation(groups, id, context)?,
        };
        if !covered {
            return Err("codec disposition contains an unowned canonical record patch".into());
        }
    }
    Ok(())
}

fn covered_entity(
    groups: &[Arc<CanonicalRequiredOutputOrigin>],
    id: crate::identity::data::EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Stop> {
    for origin in groups {
        charge(context, origin.members.len())?;
        if origin.members.iter().any(|member| {
            member.id == id && member.posture != crate::mvcc::RequiredOutputRolePosture::Retire
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}
fn covered_relation(
    groups: &[Arc<CanonicalRequiredOutputOrigin>],
    id: crate::identity::data::RelationId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Stop> {
    for origin in groups {
        charge(context, origin.relations.len())?;
        if origin.relations.iter().any(|relation| relation.id == id) {
            return Ok(true);
        }
    }
    Ok(false)
}
