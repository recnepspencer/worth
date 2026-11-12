//! Checked backing of independent intent copies, excluding shared owners.
use super::{
    CreateIntent, EntityMutationIntent, MaterializationMutationIntent, MutationIntent,
    RelationMutationIntent,
};
use crate::transactions::data::{AspectFieldPatch, EntityReference};
use std::alloc::Layout;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, FieldKey};

pub(crate) fn slots<T>(count: usize) -> Option<usize> {
    Some(Layout::array::<T>(count).ok()?.size())
}

fn reference(value: &EntityReference) -> Option<usize> {
    Some(match value {
        EntityReference::Existing(_) => 0,
        EntityReference::Created(value) => key(&value.client_key),
    })
}

fn key(value: &crate::symbols::data::ClientKey) -> usize {
    value.as_raw_str().map_or(0, str::len)
}

fn fields(value: &AspectFieldPatch) -> Option<usize> {
    value.iter().try_fold(
        slots::<(AspectFieldLocator, AspectValue)>(value.len())?,
        |bytes, (locator, value)| {
            let path = locator.field_path().fields();
            let path_bytes = path
                .iter()
                .try_fold(slots::<FieldKey>(path.len())?, |bytes, key| {
                    bytes.checked_add(key.as_str().len())
                })?;
            bytes
                .checked_add(locator.aspect().aspect_key().as_str().len())?
                .checked_add(path_bytes)?
                .checked_add(value.clone_allocation_bytes()?)
        },
    )
}

impl MutationIntent {
    /// New Vec/Box/String backing produced by this value's ordinary deep Clone.
    /// Unlike capacity telemetry, overflow refuses rather than saturating.
    pub(crate) fn copy_storage_bytes(&self) -> Option<usize> {
        use CreateIntent as C;
        use EntityMutationIntent as E;
        use MaterializationMutationIntent as M;
        use RelationMutationIntent as R;
        match self {
            Self::Create(C::Entity(value)) => {
                key(&value.client_key).checked_add(fields(&value.fields)?)
            }
            Self::Create(C::Relation(value)) => key(&value.client_key)
                .checked_add(reference(&value.source)?)?
                .checked_add(reference(&value.target)?)?
                .checked_add(fields(&value.fields)?),
            Self::Create(C::EntityAspects(value)) => {
                key(&value.client_key).checked_add(value.aspect_patch.clone_allocation_bytes()?)
            }
            Self::Create(C::RelationAspects(value)) => key(&value.client_key)
                .checked_add(reference(&value.source)?)?
                .checked_add(reference(&value.target)?)?
                .checked_add(value.aspect_patch.clone_allocation_bytes()?),
            Self::Create(C::BulkEntities(value)) => {
                let bytes = value.client_keys.iter().try_fold(
                    slots::<crate::symbols::data::ClientKey>(value.client_keys.len())?,
                    |bytes, value| bytes.checked_add(key(value)),
                )?;
                value.field_patches.iter().try_fold(
                    bytes.checked_add(slots::<AspectFieldPatch>(value.field_patches.len())?)?,
                    |bytes, value| bytes.checked_add(fields(value)?),
                )
            }
            Self::Create(C::BulkRelations(value)) => {
                let bytes = value.client_keys.iter().try_fold(
                    slots::<crate::symbols::data::ClientKey>(value.client_keys.len())?,
                    |bytes, value| bytes.checked_add(key(value)),
                )?;
                let bytes = value.endpoints.iter().try_fold(
                    bytes.checked_add(slots::<(EntityReference, EntityReference)>(
                        value.endpoints.len(),
                    )?)?,
                    |bytes, (source, target)| {
                        bytes
                            .checked_add(reference(source)?)?
                            .checked_add(reference(target)?)
                    },
                )?;
                value.field_patches.iter().try_fold(
                    bytes.checked_add(slots::<AspectFieldPatch>(value.field_patches.len())?)?,
                    |bytes, value| bytes.checked_add(fields(value)?),
                )
            }
            Self::Entity(E::UpdateFields(value)) => fields(&value.fields),
            Self::Entity(E::ApplyAspectPatch(value)) => value.aspect_patch.clone_allocation_bytes(),
            Self::Entity(E::Replace(value)) => {
                key(&value.replacement.client_key).checked_add(fields(&value.replacement.fields)?)
            }
            Self::Relation(R::UpdateEndpoints(value)) => {
                reference(&value.source)?.checked_add(reference(&value.target)?)
            }
            Self::Relation(R::ApplyAspectPatch(value)) => {
                value.aspect_patch.clone_allocation_bytes()
            }
            Self::Materialization(M::RematerializeEntity(value)) => fields(&value.fields),
            Self::Materialization(M::RematerializeRelation(value)) => fields(&value.fields),
            Self::Entity(E::Delete(_) | E::Revalidate(_))
            | Self::Relation(R::Delete(_) | R::Revalidate(_))
            | Self::Materialization(M::SuspendEntity(_) | M::SuspendRelation(_)) => Some(0),
        }
    }
}
