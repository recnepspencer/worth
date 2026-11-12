//! The exact three-field current artifact grammar uses controlled Native content ingress.
use super::ArtifactPayloadWire;
use crate::content::decoding::ContentDecodeContext;
use crate::history::data::CanonicalEnvelopeSeed;
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::Deserializer;

pub(in crate::durability::canonical_commit_artifact) fn decode_payload<'de>(
    payload: &'de [u8],
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<ArtifactPayloadWire<'de>, super::super::RelationalCanonicalCommitArtifactRefusal> {
    use super::super::RelationalCanonicalCommitArtifactRefusal as Refusal;
    let context = ContentDecodeContext::new(execution);
    let mut decoder = rmp_serde::Deserializer::from_read_ref(payload);
    let answer = ArtifactPayloadSeed(&context).deserialize(&mut decoder);
    if let Some(cause) = context.take_stop() {
        return Err(Refusal::Content(cause));
    }
    answer.map_err(|cause| Refusal::PayloadDecode {
        detail: cause.to_string(),
    })
}
struct ArtifactPayloadSeed<'a, 'r, 'c, 's>(&'a ContentDecodeContext<'r, 'c, 's>);
impl<'de> DeserializeSeed<'de> for ArtifactPayloadSeed<'_, '_, '_, '_> {
    type Value = ArtifactPayloadWire<'de>;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for ArtifactPayloadSeed<'_, '_, '_, '_> {
    type Value = ArtifactPayloadWire<'de>;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("positioned canonical envelope with original inputs")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut position = None;
        let mut canonical = None;
        let mut original_inputs = None;
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "position" => {
                    if position.is_some() {
                        return Err(M::Error::duplicate_field("position"));
                    }
                    position = Some(map.next_value()?);
                }
                "canonical" => {
                    if canonical.is_some() {
                        return Err(M::Error::duplicate_field("canonical"));
                    }
                    canonical = Some(map.next_value_seed(CanonicalEnvelopeSeed(self.0))?);
                }
                "original_inputs" => {
                    if original_inputs.is_some() {
                        return Err(M::Error::duplicate_field("original_inputs"));
                    }
                    original_inputs = Some(map.next_value()?);
                }
                _ => {
                    return Err(M::Error::unknown_field(
                        key,
                        &["position", "canonical", "original_inputs"],
                    ))
                }
            }
        }
        Ok(ArtifactPayloadWire {
            position: position.ok_or_else(|| M::Error::missing_field("position"))?,
            canonical: canonical.ok_or_else(|| M::Error::missing_field("canonical"))?,
            original_inputs: original_inputs
                .ok_or_else(|| M::Error::missing_field("original_inputs"))?,
        })
    }
}
