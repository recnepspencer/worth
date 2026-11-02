//! Persistent transaction attachments retain the exact original grants.
use super::{
    RelationalContentDenial as Denial, RelationalContentFrame, RelationalContentRecord,
    MAXIMUM_STAGED_CONTENT_BYTES,
};
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};

#[derive(Clone, Debug, Default)]
pub struct RelationalContentAttachments {
    live: Option<Arc<Staged>>,
    transport: Option<Arc<Vec<RelationalContentRecord>>>,
}
#[derive(Debug)]
struct Staged {
    records: ExecutionOrderedMap<u64, RelationalContentRecord>,
    bytes: usize,
    _memory: Option<ExecutionMemoryReservation>,
}
impl PartialEq for RelationalContentAttachments {
    fn eq(&self, other: &Self) -> bool {
        self.records().eq(other.records())
    }
}
impl Eq for RelationalContentAttachments {}
impl RelationalContentAttachments {
    pub(crate) fn same_staging(&self, other: &Self) -> bool {
        match (&self.live, &other.live) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => self.is_empty() && other.is_empty(),
            _ => false,
        }
    }
    pub fn records(&self) -> impl Iterator<Item = &RelationalContentRecord> {
        self.live
            .iter()
            .flat_map(|stage| stage.records.values())
            .chain(self.transport.iter().flat_map(|records| records.iter()))
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn len(&self) -> usize {
        self.live.as_ref().map_or(0, |stage| stage.records.len())
            + self.transport.as_ref().map_or(0, |records| records.len())
    }
    pub fn attach(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        frame: &RelationalContentFrame,
    ) -> Result<(), Denial> {
        if self.transport.is_some() {
            return Err(Denial::IntegrityMismatch);
        }
        let prior = self.live.as_ref().map_or(0, |stage| stage.bytes);
        let bytes = prior
            .checked_add(frame.bytes().len())
            .ok_or(Denial::ArithmeticOverflow)?;
        if bytes > MAXIMUM_STAGED_CONTENT_BYTES {
            return Err(Denial::SizeLimit {
                observed: bytes,
                maximum: MAXIMUM_STAGED_CONTENT_BYTES,
            });
        }
        let memory = execution
            .reserve_retained_memory((size_of::<Staged>() + 2 * size_of::<usize>()) as u64)
            .map_err(|cause| Denial::Execution(cause.into()))?;
        let mut records = match &self.live {
            Some(stage) => stage.records.clone(),
            None => ExecutionOrderedMap::default(),
        };
        let index = u64::try_from(records.len()).map_err(|_| Denial::ArithmeticOverflow)?;
        let height = u64::from(usize::BITS - records.len().leading_zeros()) * 2 + 2;
        super::control::run(execution, |control| {
            control.checkpoint(
                height * (size_of::<u64>() + size_of::<RelationalContentRecord>()) as u64,
            )
        })?;
        records
            .insert_cloned_native_for_request(&index, &frame.record(), *execution)
            .map_err(super::directory::map_denial)?;
        self.live = Some(Arc::new(Staged {
            records,
            bytes,
            _memory: memory,
        }));
        Ok(())
    }
}
impl Serialize for RelationalContentAttachments {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.len()))?;
        for record in self.records() {
            sequence.serialize_element(record)?;
        }
        sequence.end()
    }
}
impl<'de> Deserialize<'de> for RelationalContentAttachments {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = RelationalContentAttachments;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("bounded Native content attachments")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut input: A,
            ) -> Result<Self::Value, A::Error> {
                use serde::de::Error;
                let mut records = Vec::new();
                let mut bytes = 0_usize;
                while let Some(record) = input.next_element::<RelationalContentRecord>()? {
                    bytes = bytes
                        .checked_add(record.bytes().len())
                        .ok_or_else(|| A::Error::custom("content byte overflow"))?;
                    if bytes > MAXIMUM_STAGED_CONTENT_BYTES {
                        return Err(A::Error::custom("content transaction size limit"));
                    }
                    records.try_reserve(1).map_err(A::Error::custom)?;
                    records.push(record);
                }
                let transport = if records.is_empty() {
                    None
                } else {
                    Some(Arc::new(records))
                };
                Ok(RelationalContentAttachments {
                    live: None,
                    transport,
                })
            }
        }
        deserializer.deserialize_seq(Visitor)
    }
}
