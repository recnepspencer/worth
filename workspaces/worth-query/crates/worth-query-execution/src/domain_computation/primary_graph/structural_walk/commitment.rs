//! Canonical source descriptor only; original bodies still establish equality.
use super::*;
use sha2::{Digest, Sha256};
use worth_relational::facade::identity::EntityId;

pub(super) fn derive(
    query: &StructuralWalkKey,
    native: &PinnedStructuralWalk,
    context: &mut MapKernelContext<'_, '_>,
    control: Option<&dyn Fn() -> Result<(), Cause>>,
) -> Result<[u8; 32], Cause> {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:complete-structural-source:v1");
    digest.update((query.relation().len() as u64).to_be_bytes());
    digest.update(query.relation().as_bytes());
    digest.update(query.entity_kind().as_u32().to_be_bytes());
    digest.update(query.relation_kind().as_u32().to_be_bytes());
    digest.update([u8::from(
        query.direction() == worth_foundational::facade::RelationDirection::Reverse,
    )]);
    digest.update((query.roots().len() as u64).to_be_bytes());
    for root in query.roots() {
        StructuralWalkObservation::checkpoint(context)?;
        if let Some(control) = control {
            control()?;
        }
        entity(&mut digest, *root);
    }
    digest.update((native.visits().len() as u64).to_be_bytes());
    for visit in native.visits() {
        StructuralWalkObservation::checkpoint(context)?;
        if let Some(control) = control {
            control()?;
        }
        entity(&mut digest, visit.entity.entity_id);
        entity(&mut digest, visit.root);
        digest.update(visit.entity.kind_id.as_u32().to_be_bytes());
        digest.update((visit.distance as u64).to_be_bytes());
        // Only live metadata can be emitted by a successful Native walk.
        digest.update([1]);
    }
    digest.update((native.relations().len() as u64).to_be_bytes());
    for edge in native.relations() {
        StructuralWalkObservation::checkpoint(context)?;
        if let Some(control) = control {
            control()?;
        }
        digest.update(edge.relation_id.partition_value().to_be_bytes());
        digest.update(edge.relation_id.local_slot_value().to_be_bytes());
        digest.update(edge.relation_id.generation_value().to_be_bytes());
        digest.update(edge.kind_id.as_u32().to_be_bytes());
        entity(&mut digest, edge.source);
        entity(&mut digest, edge.target);
        digest.update([1]);
    }
    StructuralWalkObservation::checkpoint(context)?;
    if let Some(control) = control {
        control()?;
    }
    Ok(digest.finalize().into())
}

fn entity(digest: &mut Sha256, entity: EntityId) {
    digest.update(entity.partition_value().to_be_bytes());
    digest.update(entity.local_slot_value().to_be_bytes());
    digest.update(entity.generation_value().to_be_bytes());
}
