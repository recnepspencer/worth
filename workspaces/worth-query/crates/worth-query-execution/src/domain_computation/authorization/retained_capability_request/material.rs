//! Full capability-use meaning streamed under the caller's original checkpoint.
use super::*;
use sha2::{Digest, Sha256};
use worth_foundational::facade::{write_aspect_value_identity_material, CanonicalMaterialSink};
use worth_relational::facade::identity::EntityId;

pub(super) fn descriptor<E>(
    request: &WorthQueryRetainedCapabilityRequest,
    admit: &mut impl FnMut(u64) -> Result<(), E>,
    overflow: &dyn Fn() -> E,
) -> Result<[u8; 32], E> {
    let mut material = Material::new(admit, overflow);
    material.bytes(b"worth-query:capability-use:v1")?;
    material.bytes(&request.capability_identity)?;
    material.entity(request.principal)?;
    material.entity(request.resource)?;
    material.bytes(request.resource_entity.as_bytes())?;
    material.optional_entity(request.elevation)?;
    material.value(&request.action)?;
    material.value(&request.purpose)?;
    material.number(u64::from(request.related_relation.is_some()))?;
    if let Some(relation) = &request.related_relation {
        for text in [relation.relation(), relation.from(), relation.to()] {
            material.bytes(text.as_bytes())?;
        }
    }
    material.optional_entity(request.related)?;
    material.optional_value(request.field.as_ref())?;
    material.optional_value(request.magnitude.as_ref())?;
    material.number(u64::from(request.cardinality))?;
    material.bytes(request.context_name.as_bytes())?;
    material.bytes(request.context_type.as_bytes())?;
    // Both immutable Native and decoded representations iterate unique sorted keys.
    for (key, entity) in request.context.iter() {
        material.number(1)?;
        for text in key.meaning_parts() {
            material.bytes(text.as_bytes())?;
        }
        material.entity(*entity)?;
    }
    material.number(0)?;
    material.number(u64::from(request.recorded_actor.is_some()))?;
    if let Some(actor) = &request.recorded_actor {
        // The actor owner binds precisely the original-source provenance that
        // its full comparator consumes. This digest grants no actor authority.
        let mut digest = Sha256::new();
        actor.visit_identity_material(&[0; 32], &mut |bytes| {
            (material.admit)(bytes.len() as u64)?;
            digest.update(bytes);
            Ok(())
        })?;
        material.bytes(&digest.finalize())?;
    }
    (material.admit)(0)?;
    Ok(material.finish())
}

/// Local framing belongs to this versioned request descriptor; scalar meaning
/// continues to use the sole Foundational borrowed canonical encoder.
pub(in crate::domain_computation::authorization) struct Material<'a, E> {
    digest: Sha256,
    admit: &'a mut dyn FnMut(u64) -> Result<(), E>,
    overflow: &'a dyn Fn() -> E,
}
impl<'a, E> Material<'a, E> {
    pub(in crate::domain_computation::authorization) fn new(
        admit: &'a mut dyn FnMut(u64) -> Result<(), E>,
        overflow: &'a dyn Fn() -> E,
    ) -> Self {
        Self {
            digest: Sha256::new(),
            admit,
            overflow,
        }
    }
    pub(in crate::domain_computation::authorization) fn bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), E> {
        (self.admit)(8)?;
        self.digest.update((bytes.len() as u64).to_be_bytes());
        for chunk in bytes.chunks(4096) {
            (self.admit)(chunk.len() as u64)?;
            self.digest.update(chunk);
        }
        Ok(())
    }
    fn number(&mut self, number: u64) -> Result<(), E> {
        self.bytes(&number.to_be_bytes())
    }
    fn entity(&mut self, entity: EntityId) -> Result<(), E> {
        self.number(entity.partition_value_u64())?;
        self.number(entity.local_slot_value())?;
        self.number(u64::from(entity.generation_value()))
    }
    fn optional_entity(&mut self, value: Option<EntityId>) -> Result<(), E> {
        self.number(u64::from(value.is_some()))?;
        if let Some(value) = value {
            self.entity(value)?;
        }
        Ok(())
    }
    fn optional_value(&mut self, value: Option<&AspectValue>) -> Result<(), E> {
        self.number(u64::from(value.is_some()))?;
        if let Some(value) = value {
            self.value(value)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::authorization) fn value(
        &mut self,
        value: &AspectValue,
    ) -> Result<(), E> {
        let digest = {
            let mut scalar = Material::new(self.admit, self.overflow);
            write_aspect_value_identity_material(value, &mut scalar)?;
            scalar.finish()
        };
        self.bytes(&digest)
    }
    pub(in crate::domain_computation::authorization) fn finish(self) -> [u8; 32] {
        self.digest.finalize().into()
    }
}
impl<E> CanonicalMaterialSink for Material<'_, E> {
    type Error = E;
    fn append(&mut self, text: &str) -> Result<(), E> {
        for chunk in text.as_bytes().chunks(4096) {
            (self.admit)(chunk.len() as u64)?;
            self.digest.update(chunk);
        }
        Ok(())
    }
    fn admit_work(&mut self, work: usize) -> Result<(), E> {
        (self.admit)(work as u64)
    }
    fn accounting_overflow(&mut self) -> E {
        (self.overflow)()
    }
}

#[cfg(test)]
mod tests;
