//! An immutable projection retains its digest and original backing together.
use super::SchemaAuthoritySnapshotParts;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone)]
pub struct SchemaAuthoritySnapshot(Arc<Projection>);

#[derive(Debug)]
struct Projection {
    parts: SchemaAuthoritySnapshotParts,
    digest: OnceLock<[u8; 32]>,
    owned_bytes: OnceLock<u64>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl SchemaAuthoritySnapshot {
    pub(crate) fn shares_projection_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn projection_allocation_bytes(
    ) -> Result<u64, crate::execution::RelationalExecutionDenialCause> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Projection>())
            .map_err(|_| crate::execution::RelationalExecutionDenialCause::MemoryOverflow)?
            .0
            .pad_to_align();
        u64::try_from(layout.size())
            .map_err(|_| crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
    }

    pub(crate) fn from_compiled_parts(
        parts: SchemaAuthoritySnapshotParts,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self(Arc::new(Projection {
            parts,
            digest: OnceLock::new(),
            owned_bytes: OnceLock::new(),
            _memory: memory,
        }))
    }

    pub(super) fn digest_for_compilation(
        &self,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<[u8; 32], crate::execution::RelationalExecutionDenialCause> {
        control.checkpoint()?;
        if let Some(digest) = self.0.digest.get() {
            return Ok(*digest);
        }
        let digest = super::digest_parts_for_compilation(&self.0.parts, control)?;
        let _same_immutable_digest = self.0.digest.set(digest);
        Ok(digest)
    }

    pub fn digest(&self) -> [u8; 32] {
        *self
            .0
            .digest
            .get_or_init(|| super::digest_parts(&self.0.parts))
    }

    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        *self
            .0
            .owned_bytes
            .get_or_init(|| self.0.parts.owned_allocation_capacity_bytes())
    }
}

impl From<SchemaAuthoritySnapshotParts> for SchemaAuthoritySnapshot {
    fn from(parts: SchemaAuthoritySnapshotParts) -> Self {
        Self::from_compiled_parts(parts, None)
    }
}
impl Deref for SchemaAuthoritySnapshot {
    type Target = SchemaAuthoritySnapshotParts;
    fn deref(&self) -> &Self::Target {
        &self.0.parts
    }
}
impl PartialEq for SchemaAuthoritySnapshot {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.digest() == other.digest()
    }
}
impl Eq for SchemaAuthoritySnapshot {}
impl Serialize for SchemaAuthoritySnapshot {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.parts.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for SchemaAuthoritySnapshot {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        SchemaAuthoritySnapshotParts::deserialize(deserializer).map(Self::from)
    }
}
