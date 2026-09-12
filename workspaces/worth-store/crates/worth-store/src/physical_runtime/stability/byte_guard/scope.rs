use crate::physical_runtime::{PhysicalRecordChunkBasis, PhysicalRecordChunkView};
use worth_store_physical_isolation::CurrentGenerationPhysicalReference;

/// Scope is derived from a Store-issued chunk, never paired with an arbitrary reference.
///
/// ```compile_fail,E0061
/// use worth_store::physical_runtime::{PhysicalRecordChunkView, stability::PhysicalByteGuardScope};
/// use worth_store_physical_isolation::CurrentGenerationPhysicalReference;
/// fn pair(reference: CurrentGenerationPhysicalReference, chunk: &PhysicalRecordChunkView<'_>) {
///     let _ = PhysicalByteGuardScope::for_record_chunk(reference, chunk);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalByteGuardScope {
    reference: CurrentGenerationPhysicalReference,
    chunk_basis: PhysicalRecordChunkBasis,
}

impl PhysicalByteGuardScope {
    pub fn for_record_chunk(chunk: &PhysicalRecordChunkView<'_>) -> Self {
        Self {
            reference: super::current_reference_for_record_chunk(chunk),
            chunk_basis: chunk.basis(),
        }
    }

    pub const fn reference(self) -> CurrentGenerationPhysicalReference {
        self.reference
    }

    pub const fn chunk_basis(self) -> PhysicalRecordChunkBasis {
        self.chunk_basis
    }
}
