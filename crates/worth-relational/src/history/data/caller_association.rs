//! Opaque caller meaning retained by Native's canonical envelope grammar.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

/// Immutable caller-defined bytes. Ordinary bytes grant no publication,
/// recovery or completion authority. The reserved Home-step protocol additionally
/// requires the attached Home owner's concrete step grant before preparation.
///
/// ```compile_fail,E0423
/// use worth_relational::facade::mvcc::RelationalCallerAssociation;
/// let fabricated = RelationalCallerAssociation(std::sync::Arc::from(&b"caller"[..]));
/// ```
#[derive(Clone)]
pub struct RelationalCallerAssociation {
    bytes: Arc<[u8]>,
    // Last: discovery/index/subject clones keep the same original byte payer.
    decoded_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}
impl std::fmt::Debug for RelationalCallerAssociation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalCallerAssociation")
            .field("bytes", &self.bytes)
            .field("decoded_funding", &self.decoded_memory.is_some())
            .finish()
    }
}
impl PartialEq for RelationalCallerAssociation {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}
impl Eq for RelationalCallerAssociation {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalCallerAssociationRefusal {
    Empty,
    TooLong { actual: usize },
}

impl RelationalCallerAssociation {
    pub const MAX_LEN: usize = 4096;

    pub(crate) fn payload_allocation_bytes(count: usize) -> Option<u64> {
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::array::<u8>(count).ok()?)
            .ok()?
            .0
            .pad_to_align()
            .size();
        u64::try_from(bytes).ok()
    }

    pub(crate) fn retained_allocation_bytes(count: usize) -> Option<u64> {
        let bytes = Self::payload_allocation_bytes(count)?;
        let funding = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<
                worth_execution::ExecutionMemoryReservation,
            >())
            .ok()?
            .0
            .pad_to_align()
            .size();
        bytes.checked_add(u64::try_from(funding).ok()?)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, RelationalCallerAssociationRefusal> {
        if bytes.is_empty() {
            return Err(RelationalCallerAssociationRefusal::Empty);
        }
        if bytes.len() > Self::MAX_LEN {
            return Err(RelationalCallerAssociationRefusal::TooLong {
                actual: bytes.len(),
            });
        }
        Ok(Self {
            bytes: Arc::from(bytes),
            decoded_memory: None,
        })
    }

    pub(crate) fn with_decoded_memory(
        mut self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        self.decoded_memory = memory.map(Arc::new);
        self
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Serialize for RelationalCallerAssociation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl<'de> Deserialize<'de> for RelationalCallerAssociation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Bytes;
        impl serde::de::Visitor<'_> for Bytes {
            type Value = RelationalCallerAssociation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a nonempty caller association of at most 4096 bytes")
            }

            fn visit_bytes<E: serde::de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                RelationalCallerAssociation::from_bytes(bytes)
                    .map_err(|refusal| E::custom(format!("caller association: {refusal:?}")))
            }
        }
        deserializer.deserialize_bytes(Bytes)
    }
}
