//! Required-output meaning retains logical source references, never runtime slots.
use super::reference;
use crate::history::data::{
    CanonicalCommitEnvelope, CanonicalRequiredOutputCodecRetirement, CanonicalRequiredOutputOrigin,
};
use serde::{
    ser::{SerializeSeq, SerializeStruct},
    Serialize, Serializer,
};

pub(super) struct Projection<'a>(pub(super) &'a CanonicalCommitEnvelope);
impl Serialize for Projection<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut record = serializer.serialize_struct("CheckpointRequiredOutputs", 3)?;
        record.serialize_field(
            "origin",
            &self.0.required_output_origin.as_deref().map(Origin),
        )?;
        record.serialize_field("materialization", &self.0.required_output_materialization)?;
        record.serialize_field(
            "retirements",
            &Retirements(&self.0.required_output_codec_retirements),
        )?;
        record.end()
    }
}

struct Origin<'a>(&'a CanonicalRequiredOutputOrigin);
impl Serialize for Origin<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let source = self.0;
        let mut record = serializer.serialize_struct("CheckpointRequiredOutputOrigin", 12)?;
        record.serialize_field("commit", &source.commit)?;
        record.serialize_field("basis", &reference::component(&source.basis))?;
        record.serialize_field("producer", &source.producer)?;
        record.serialize_field("dependency_identity", &source.dependency_identity)?;
        record.serialize_field("outputs", &source.outputs)?;
        record.serialize_field("relations", &source.relations)?;
        record.serialize_field("meaning", &source.meaning)?;
        record.serialize_field("publication", &source.publication)?;
        record.serialize_field("birth", &source.birth)?;
        record.serialize_field("birth_producer", &source.birth_producer)?;
        record.serialize_field("birth_dependency", &source.birth_dependency)?;
        record.serialize_field("members", &source.members)?;
        record.end()
    }
}

struct Retirements<'a>(&'a [std::sync::Arc<CanonicalRequiredOutputCodecRetirement>]);
impl Serialize for Retirements<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for source in self.0 {
            sequence.serialize_element(&Retirement(source))?;
        }
        sequence.end()
    }
}
struct Retirement<'a>(&'a CanonicalRequiredOutputCodecRetirement);
impl Serialize for Retirement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let source = self.0;
        let mut record =
            serializer.serialize_struct("CheckpointRequiredOutputCodecRetirement", 6)?;
        record.serialize_field("publication", &source.publication)?;
        record.serialize_field("meaning", &source.meaning)?;
        record.serialize_field("source", &reference::component(&source.source))?;
        record.serialize_field("retiring", &source.retiring)?;
        record.serialize_field("kinds", &source.kinds)?;
        record.serialize_field("activation", &source.activation)?;
        record.end()
    }
}

#[cfg(test)]
mod tests;
