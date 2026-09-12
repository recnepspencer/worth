use crate::physical_runtime::PhysicalRecordChunkView;
use worth_store_physical_isolation::{
    CurrentGenerationPhysicalReference, GenerationCountedPhysicalReference,
};

/// Describe the exact generation already carried by a Store-issued chunk.
pub fn current_reference_for_record_chunk(
    chunk: &PhysicalRecordChunkView<'_>,
) -> CurrentGenerationPhysicalReference {
    let owner = chunk.basis().physical_owner();
    GenerationCountedPhysicalReference::from_durable_owner(owner)
        .expect("Store record chunks carry a generation-counted physical owner")
        .require_current_generation(owner.generation())
        .expect("the reference preserves the chunk owner's generation")
}
