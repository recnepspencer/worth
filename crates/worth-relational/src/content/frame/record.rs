//! Serializable bytes are descriptive; recovery must run Native admission again.
use super::super::{RelationalContentDenial as Denial, MAXIMUM_CONTENT_FRAME_BYTES};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::ContentRefId;

#[derive(Clone, Debug)]
pub struct RelationalContentRecord {
    pub(super) identity: ContentRefId,
    pub(super) backing: Arc<FrameBacking>,
}

#[derive(Debug)]
pub(super) struct FrameBacking {
    pub(super) bytes: Vec<u8>,
    pub(super) memory: Option<ExecutionMemoryReservation>,
}

impl RelationalContentRecord {
    pub fn from_untrusted_bytes(identity: ContentRefId, bytes: Vec<u8>) -> Result<Self, Denial> {
        super::layout::admit_length(bytes.len(), MAXIMUM_CONTENT_FRAME_BYTES)?;
        if bytes.len() < 25 {
            return Err(Denial::NonCanonicalContent);
        }
        Ok(Self {
            identity,
            backing: Arc::new(FrameBacking {
                bytes,
                memory: None,
            }),
        })
    }
    pub fn identity(&self) -> ContentRefId {
        self.identity
    }
    pub fn bytes(&self) -> &[u8] {
        &self.backing.bytes
    }
}

impl PartialEq for RelationalContentRecord {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity && self.bytes() == other.bytes()
    }
}
impl Eq for RelationalContentRecord {}

impl Serialize for RelationalContentRecord {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.identity, self.bytes()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RelationalContentRecord {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BoundedBytes(Vec<u8>);
        impl<'de> Deserialize<'de> for BoundedBytes {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct Visitor;
                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = BoundedBytes;
                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        formatter.write_str("a Native content frame within its fixed size ceiling")
                    }
                    fn visit_seq<A: serde::de::SeqAccess<'de>>(
                        self,
                        mut input: A,
                    ) -> Result<Self::Value, A::Error> {
                        use serde::de::Error;
                        if input
                            .size_hint()
                            .is_some_and(|size| size > MAXIMUM_CONTENT_FRAME_BYTES)
                        {
                            return Err(A::Error::custom("content frame size limit"));
                        }
                        let mut bytes = Vec::new();
                        while let Some(byte) = input.next_element::<u8>()? {
                            if bytes.len() == MAXIMUM_CONTENT_FRAME_BYTES {
                                return Err(A::Error::custom("content frame size limit"));
                            }
                            bytes.try_reserve(1).map_err(A::Error::custom)?;
                            bytes.push(byte);
                        }
                        Ok(BoundedBytes(bytes))
                    }
                }
                deserializer.deserialize_seq(Visitor)
            }
        }
        let (identity, bytes) = <(ContentRefId, BoundedBytes)>::deserialize(deserializer)?;
        Self::from_untrusted_bytes(identity, bytes.0).map_err(serde::de::Error::custom)
    }
}
