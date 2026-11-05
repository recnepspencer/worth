//! Compose each record at its first authored patch, preserving other intent order.
use super::*;
use std::collections::{btree_map::Entry, BTreeMap};
use worth_foundational::facade::PortableRecordAspectPatch;
use worth_relational::facade::{
    identity::EntityId,
    transactions::{ApplyEntityAspectPatchIntent, EntityMutationIntent},
};

enum AuthoredIntent {
    Mutation(MutationIntent),
    AspectPatch(EntityId),
}

// Grouping E effects, R patched records and P patch operations costs
// O(E log(R + 1) + P) work and O(E + P) grouping material. Expected-step
// validation separately owns its S-step cost. Payloads move without cloning;
// unrelated intents are never sorted here.
pub(super) fn materialize_commit_projections(
    lowered: Vec<WorthQueryLoweredProviderEffect>,
) -> Result<
    (
        Vec<MutationIntent>,
        Vec<WorthQueryApplicationEmission>,
        WorthQueryExpectedEffectSteps,
    ),
    WorthQueryApplicationAttemptDenial,
> {
    let mut authored = Vec::new();
    let mut emissions = Vec::new();
    let mut patches = BTreeMap::<_, Vec<_>>::new();
    let expected_steps = WorthQueryExpectedEffectSteps::from_step_groups(
        lowered.into_iter().filter_map(|effect| match effect {
            WorthQueryLoweredProviderEffect::EntityAspectPatch {
                steps,
                entity_id,
                operations,
            } => {
                match patches.entry(entity_id) {
                    Entry::Vacant(entry) => {
                        authored.push(AuthoredIntent::AspectPatch(entity_id));
                        entry.insert(operations);
                    }
                    Entry::Occupied(mut entry) => entry.get_mut().extend(operations),
                }
                Some(steps)
            }
            WorthQueryLoweredProviderEffect::Mutation { intent, steps } => {
                authored.push(AuthoredIntent::Mutation(intent));
                Some(steps)
            }
            WorthQueryLoweredProviderEffect::ObservedNoop { steps } => Some(steps),
            WorthQueryLoweredProviderEffect::Emission(emission) => {
                emissions.push(emission);
                None
            }
        }),
    )?;
    let intents = authored.into_iter().map(|intent| match intent {
        AuthoredIntent::Mutation(intent) => Ok(intent),
        AuthoredIntent::AspectPatch(entity_id) => {
            let operations = patches.remove(&entity_id).ok_or_else(|| {
                WorthQueryApplicationAttemptDenial::new(
                    crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind::IncompleteEffectBasis,
                    "authored aspect patch lost its composed material",
                )
            })?;
            Ok(MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(
                ApplyEntityAspectPatchIntent {
                    entity_id,
                    aspect_patch: PortableRecordAspectPatch::new(operations),
                },
            )))
        }
    }).collect::<Result<Vec<_>, WorthQueryApplicationAttemptDenial>>()?;
    Ok((intents, emissions, expected_steps))
}
