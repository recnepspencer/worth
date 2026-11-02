//! Complete one-hop projected content, never a revision or equality witness.
use super::*;
use sha2::{Digest, Sha256};
use worth_foundational::facade::{write_aspect_value_identity_material, CanonicalMaterialSink};

pub(in crate::domain_computation::primary_graph::application_query) struct ContentWriter<'a, E> {
    digest: Sha256,
    admit: &'a mut dyn FnMut(u64) -> Result<(), E>,
    overflow: &'a dyn Fn() -> E,
}
impl<'a, E> ContentWriter<'a, E> {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        admit: &'a mut dyn FnMut(u64) -> Result<(), E>,
        overflow: &'a dyn Fn() -> E,
    ) -> Self {
        Self {
            digest: Sha256::new(),
            admit,
            overflow,
        }
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn bytes(
        &mut self,
        value: &[u8],
    ) -> Result<(), E> {
        // No concatenated text or unmetered sizing pass. Chunk boundaries are
        // framed for structural data; scalar canonical material has its own hash.
        (self.admit)(8)?;
        self.digest.update((value.len() as u64).to_be_bytes());
        for chunk in value.chunks(4096) {
            (self.admit)(chunk.len() as u64)?;
            self.digest.update(chunk);
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn number(
        &mut self,
        value: u64,
    ) -> Result<(), E> {
        self.bytes(&value.to_be_bytes())
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn entity(
        &mut self,
        value: EntityId,
    ) -> Result<(), E> {
        self.number(value.partition_value_u64())?;
        self.number(value.local_slot_value())?;
        self.number(u64::from(value.generation_value()))
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn finish(
        self,
    ) -> [u8; 32] {
        self.digest.finalize().into()
    }
    fn scalar(&mut self, value: &AspectValue) -> Result<(), E> {
        let digest: [u8; 32] = {
            let mut scalar = ScalarWriter {
                digest: Sha256::new(),
                admit: self.admit,
                overflow: self.overflow,
            };
            write_aspect_value_identity_material(value, &mut scalar)?;
            scalar.digest.finalize().into()
        };
        self.bytes(&digest)
    }
    fn row(&mut self, row: &WorthQueryApplicationProjectionNode) -> Result<(), E> {
        self.entity(row.entity_id)?;
        self.bytes(row.result_path().as_bytes())?;
        self.number(u64::from(row.source_dependencies_complete))?;
        self.number(u64::from(row.incoming_relation.is_some()))?;
        if let Some(relation) = row.incoming_relation {
            self.number(relation.partition_value_u64())?;
            self.number(relation.local_slot_value())?;
            self.number(u64::from(relation.generation_value()))?;
        }
        self.number(row.fields.len() as u64)?;
        for field in &row.fields {
            self.bytes(field.result_path.as_bytes())?;
            self.bytes(field.slot_type.as_bytes())?;
            self.bytes(field.slot_key.output_name().unwrap_or("").as_bytes())?;
            self.scalar(&field.value)?;
        }
        Ok(())
    }
}
struct ScalarWriter<'a, E> {
    digest: Sha256,
    admit: &'a mut dyn FnMut(u64) -> Result<(), E>,
    overflow: &'a dyn Fn() -> E,
}
impl<E> CanonicalMaterialSink for ScalarWriter<'_, E> {
    type Error = E;
    fn append(&mut self, value: &str) -> Result<(), E> {
        for chunk in value.as_bytes().chunks(4096) {
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
impl WorthQueryApplicationDisclosedProjectionTree {
    pub(in crate::domain_computation::primary_graph::application_query) fn append_one_hop_content<
        E,
    >(
        &self,
        writer: &mut ContentWriter<'_, E>,
        unsupported: impl Fn() -> E,
    ) -> Result<(), E> {
        writer.number(self.raw_rows().len() as u64)?;
        for row in self.raw_rows() {
            writer.row(row)?;
            writer.number(row.relations.len() as u64)?;
            for relation in &row.relations {
                writer.bytes(relation.result_path.as_bytes())?;
                writer.bytes(relation.slot_type.as_bytes())?;
                writer.bytes(relation.slot_key.output_name().unwrap_or("").as_bytes())?;
                writer.number(match relation.cardinality {
                    ApplicationQueryCardinality::ExactlyOne => 1,
                    ApplicationQueryCardinality::OptionalOne => 2,
                    ApplicationQueryCardinality::Many => 3,
                })?;
                writer.number(relation.predicate_sources.len() as u64)?;
                for entity in &relation.predicate_sources {
                    writer.entity(*entity)?;
                }
                writer.number(relation.rows.len() as u64)?;
                for child in &relation.rows {
                    // Source compiler accepts direct endpoints only. Refuse
                    // future deeper shapes rather than silently omitting them.
                    if !child.relations.is_empty() {
                        return Err(unsupported());
                    }
                    writer.row(child)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
