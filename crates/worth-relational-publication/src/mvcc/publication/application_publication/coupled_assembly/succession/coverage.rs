//! What a chain round writes and publishes, as the units and roles the
//! succession proof compares.
use super::super::refusals::overflow;
use contracts::identity::data::record_ids::EntityId;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use contracts::transactions::data::intents::mutation_intent::CreateIntent;
use contracts::transactions::data::intents::mutation_intent::EntityMutationIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::EntityReference;
use std::collections::BTreeSet;
use worth_foundational::facade::{AspectKey, CanonicalFieldPath, PortableAspectPatchOperation};
use worth_relational_contracts::internal_port as contracts;

/// One coverage unit: an entity aspect's field path, or the whole aspect.
pub(super) type Unit = (EntityId, AspectKey, Option<CanonicalFieldPath>);

/// The entity an update or aspect patch writes.
pub(super) fn entity_write(intent: &MutationIntent) -> Option<EntityId> {
    match intent {
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(update)) => {
            Some(update.entity_id)
        }
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(patch)) => {
            Some(patch.entity_id)
        }
        _ => None,
    }
}

/// What `intent` writes on `id`, as coverage units.
pub(super) fn units(id: EntityId, intent: &MutationIntent, into: &mut BTreeSet<Unit>) {
    let fields = match intent {
        MutationIntent::Create(CreateIntent::Entity(spec)) => &spec.fields,
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(update)) => &update.fields,
        MutationIntent::Entity(EntityMutationIntent::ApplyAspectPatch(patch)) => {
            for operation in patch.aspect_patch.operations() {
                match operation {
                    PortableAspectPatchOperation::SetWhole { basis, .. }
                    | PortableAspectPatchOperation::ClearWhole { basis } => {
                        into.insert((id, basis.key().clone(), None));
                    }
                    PortableAspectPatchOperation::PatchFields {
                        basis,
                        selected_fields,
                        field_sets,
                        field_clears,
                    } => {
                        let fields = selected_fields
                            .iter()
                            .chain(field_sets.iter().map(|set| set.field()))
                            .chain(field_clears);
                        into.extend(fields.map(|field| {
                            let path = CanonicalFieldPath::single(field.clone());
                            (id, basis.key().clone(), Some(path))
                        }));
                    }
                }
            }
            return;
        }
        _ => return,
    };
    into.extend(fields.locators().map(|locator| {
        let key = locator.aspect().aspect_key().clone();
        (id, key, Some(locator.field_path().clone()))
    }));
}

/// A unit is covered by a write of its whole aspect or of any prefix of its path.
pub(super) fn covered(
    written: &BTreeSet<Unit>,
    id: EntityId,
    key: &AspectKey,
    path: Option<&CanonicalFieldPath>,
) -> bool {
    written.contains(&(id, key.clone(), None))
        || path.is_some_and(|path| {
            (1..=path.fields().len()).any(|end| {
                let prefix = CanonicalFieldPath::new(path.fields()[..end].iter().cloned());
                prefix.is_some_and(|prefix| written.contains(&(id, key.clone(), Some(prefix))))
            })
        })
}

/// One canonical role: name, entity name, kind, posture, actual record.
pub(super) type Role<'a> = (
    &'a str,
    &'a str,
    contracts::identity::data::primitives::KindId,
    u8,
    EntityId,
);

/// A posture's roles as one canonical set over actual records; `None` when a
/// created target has no allocation.
pub(super) fn roles<'a>(
    bindings: &contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    roles: &'a [RequiredOutputRoleTarget],
    charge: &mut impl FnMut(u64) -> Result<(), TransactionCommitError>,
) -> Result<Option<BTreeSet<Role<'a>>>, TransactionCommitError> {
    let mut set = BTreeSet::new();
    for role in roles {
        charge((role.role.len() + role.entity_name.len()) as u64 + 1)?;
        let target = match &role.target {
            EntityReference::Existing(id) => *id,
            EntityReference::Created(created) => {
                charge(bindings.resolve_work_bound(created).ok_or_else(overflow)?)?;
                match bindings.resolve(created) {
                    Some(id) => id,
                    None => return Ok(None),
                }
            }
        };
        set.insert((
            role.role.as_str(),
            role.entity_name.as_str(),
            role.kind,
            role.posture as u8,
            target,
        ));
    }
    Ok(Some(set))
}
