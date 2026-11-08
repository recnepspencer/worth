//! A projection's digest and backing travel together. Mutation detaches them.
use super::SchemaAuthoritySnapshotParts;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone)]
pub struct SchemaAuthoritySnapshot(Arc<Projection>);

#[derive(Debug, Clone)]
struct Projection {
    parts: SchemaAuthoritySnapshotParts,
    digest: OnceLock<[u8; 32]>,
    owned_bytes: OnceLock<u64>,
}

impl SchemaAuthoritySnapshot {
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
        Self(Arc::new(Projection {
            parts,
            digest: OnceLock::new(),
            owned_bytes: OnceLock::new(),
        }))
    }
}
impl Deref for SchemaAuthoritySnapshot {
    type Target = SchemaAuthoritySnapshotParts;
    fn deref(&self) -> &Self::Target {
        &self.0.parts
    }
}
impl DerefMut for SchemaAuthoritySnapshot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let projection = Arc::make_mut(&mut self.0);
        projection.digest.take();
        projection.owned_bytes.take();
        &mut projection.parts
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
