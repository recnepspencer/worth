//! Full canonical support descriptor is a rejection prefilter, never positive proof.
use super::*;
use sha2::{Digest, Sha256};
use worth_relational::facade::identity::EntityId;
pub(super) fn derive(
    key: &DerivedSourceKey,
    data: &DerivedRelationData,
    context: &mut MapKernelContext<'_, '_>,
    control: Option<&dyn Fn() -> Result<(), Cause>>,
) -> Result<[u8; 32], Cause> {
    let mut poll = || {
        DerivedSourceObservation::checkpoint(context)?;
        if let Some(control) = control {
            control()?;
        }
        Ok::<_, Cause>(())
    };
    let mut d = Sha256::new();
    d.update(b"worth-query:complete-derived-source:v1");
    for text in [
        key.relation(),
        &key.identity().schema.0,
        &key.identity().name,
    ] {
        poll()?;
        d.update((text.len() as u64).to_be_bytes());
        for byte in text.bytes() {
            poll()?;
            d.update([byte]);
        }
    }
    d.update(key.identity().version.0.to_be_bytes());
    entity(&mut d, key.root());
    d.update([u8::from(
        key.direction() == worth_foundational::facade::RelationDirection::Reverse,
    )]);
    d.update((data.endpoints().len() as u64).to_be_bytes());
    for member in data.endpoints() {
        poll()?;
        entity(
            &mut d,
            member
                .entity()
                .ok_or(Cause::ExpectedIdentitiesNotCanonical)?,
        );
    }
    d.update((data.entity_dependencies().len() as u64).to_be_bytes());
    for (id, kind) in data.entity_dependencies() {
        poll()?;
        entity(&mut d, *id);
        d.update(kind.as_u32().to_be_bytes());
    }
    d.update((data.physical_relations().len() as u64).to_be_bytes());
    for record in data.physical_relations() {
        poll()?;
        d.update(record.relation_id.partition_value().to_be_bytes());
        d.update(record.relation_id.local_slot_value().to_be_bytes());
        d.update(record.relation_id.generation_value().to_be_bytes());
        d.update(record.kind_id.as_u32().to_be_bytes());
        entity(&mut d, record.source);
        entity(&mut d, record.target);
    }
    d.update((data.adjacency_dependencies().len() as u64).to_be_bytes());
    for probe in data.adjacency_dependencies() {
        poll()?;
        entity(&mut d, probe.anchor);
        d.update(probe.relation.as_u32().to_be_bytes());
        d.update([u8::from(probe.outgoing)]);
    }
    poll()?;
    Ok(d.finalize().into())
}
fn entity(d: &mut Sha256, id: EntityId) {
    d.update(id.partition_value().to_be_bytes());
    d.update(id.local_slot_value().to_be_bytes());
    d.update(id.generation_value().to_be_bytes());
}
