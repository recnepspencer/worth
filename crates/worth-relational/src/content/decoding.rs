//! Original-request content ingress; serde strings never replace typed refusals.
use super::{
    ContentPublication, RelationalContentAttachments, RelationalContentDenial as Denial,
    RelationalContentRecord, MAXIMUM_STAGED_CONTENT_BYTES,
};
use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde::Deserializer;
use std::cell::{Cell, RefCell};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::ContentRefId;

pub(crate) struct ContentDecodeContext<'r, 'c, 's> {
    pub(crate) execution: &'r ExecutionRequest<'c, 's>,
    stop: RefCell<Option<Denial>>,
    bytes: Cell<usize>,
}
impl<'r, 'c, 's> ContentDecodeContext<'r, 'c, 's> {
    pub(crate) fn new(execution: &'r ExecutionRequest<'c, 's>) -> Self {
        Self {
            execution,
            stop: RefCell::new(None),
            bytes: Cell::new(0),
        }
    }
    pub(crate) fn take_stop(&self) -> Option<Denial> {
        self.stop.borrow_mut().take()
    }
    fn admit<T, E: Error>(&self, result: Result<T, Denial>) -> Result<T, E> {
        result.map_err(|cause| {
            let mut first = self.stop.borrow_mut();
            if first.is_none() {
                *first = Some(cause);
            }
            E::custom("Native content admission refused")
        })
    }
    fn reserve_frame(&self, length: usize) -> Result<(), Denial> {
        let bytes = self
            .bytes
            .get()
            .checked_add(length)
            .ok_or(Denial::ArithmeticOverflow)?;
        if bytes > MAXIMUM_STAGED_CONTENT_BYTES {
            return Err(Denial::SizeLimit {
                observed: bytes,
                maximum: MAXIMUM_STAGED_CONTENT_BYTES,
            });
        }
        self.bytes.set(bytes);
        Ok(())
    }
}
pub(crate) struct ContentPublicationSeed<'a, 'r, 'c, 's>(
    pub(crate) &'a ContentDecodeContext<'r, 'c, 's>,
);
impl<'de> DeserializeSeed<'de> for ContentPublicationSeed<'_, '_, '_, '_> {
    type Value = ContentPublication;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for ContentPublicationSeed<'_, '_, '_, '_> {
    type Value = ContentPublication;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("canonical Native content publication")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut records = None;
        while let Some(key) = map.next_key::<&str>()? {
            if key != "records" {
                return Err(M::Error::unknown_field(key, &["records"]));
            }
            if records.is_some() {
                return Err(M::Error::duplicate_field("records"));
            }
            records = Some(map.next_value_seed(AttachmentsSeed(self.0))?);
        }
        Ok(ContentPublication::from_readmitted_records(
            records.ok_or_else(|| M::Error::missing_field("records"))?,
        ))
    }
}
struct AttachmentsSeed<'a, 'r, 'c, 's>(&'a ContentDecodeContext<'r, 'c, 's>);
impl<'de> DeserializeSeed<'de> for AttachmentsSeed<'_, '_, '_, '_> {
    type Value = RelationalContentAttachments;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(self)
    }
}
impl<'de> Visitor<'de> for AttachmentsSeed<'_, '_, '_, '_> {
    type Value = RelationalContentAttachments;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("original-funded Native content records")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
        let mut records = RelationalContentAttachments::default();
        while let Some(record) = input.next_element_seed(RecordSeed(self.0))? {
            self.0
                .admit(records.append_record(self.0.execution, &record))?;
        }
        Ok(records)
    }
}
struct RecordSeed<'a, 'r, 'c, 's>(&'a ContentDecodeContext<'r, 'c, 's>);
impl<'de> DeserializeSeed<'de> for RecordSeed<'_, '_, '_, '_> {
    type Value = RelationalContentRecord;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_tuple(2, self)
    }
}
impl<'de> Visitor<'de> for RecordSeed<'_, '_, '_, '_> {
    type Value = RelationalContentRecord;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("content identity and borrowed binary frame")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut input: A) -> Result<Self::Value, A::Error> {
        let identity = input
            .next_element::<ContentRefId>()?
            .ok_or_else(|| A::Error::invalid_length(0, &self))?;
        let frame = input
            .next_element::<&'de [u8]>()?
            .ok_or_else(|| A::Error::invalid_length(1, &self))?;
        if input.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(A::Error::invalid_length(3, &self));
        }
        self.0.admit(self.0.reserve_frame(frame.len()))?;
        self.0.admit(RelationalContentRecord::readmit_transport(
            self.0.execution,
            identity,
            frame,
        ))
    }
}
