//! Opaque caller meaning retained by Native's canonical envelope grammar.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;

/// Immutable caller-defined bytes. Native preserves them without interpreting
/// them or granting publication, recovery, or completion authority.
///
/// ```compile_fail,E0423
/// use worth_relational::facade::mvcc::RelationalCallerAssociation;
/// let fabricated = RelationalCallerAssociation(std::sync::Arc::from(&b"caller"[..]));
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalCallerAssociation(Arc<[u8]>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalCallerAssociationRefusal {
    Empty,
    TooLong { actual: usize },
}

impl RelationalCallerAssociation {
    pub const MAX_LEN: usize = 4096;

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, RelationalCallerAssociationRefusal> {
        if bytes.is_empty() {
            return Err(RelationalCallerAssociationRefusal::Empty);
        }
        if bytes.len() > Self::MAX_LEN {
            return Err(RelationalCallerAssociationRefusal::TooLong {
                actual: bytes.len(),
            });
        }
        Ok(Self(Arc::from(bytes)))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
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
