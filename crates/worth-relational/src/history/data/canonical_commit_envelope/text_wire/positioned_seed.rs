//! Shared grammar for a positioned canonical envelope, with controlled content ingress.
use super::{CanonicalEnvelopeSeed, CanonicalEnvelopeWire};
use crate::content::decoding::ContentDecodeContext;
use crate::publication::patch::data::PatchStreamPosition;
use serde::de::{DeserializeSeed, Error, MapAccess, Visitor};
use serde::Deserializer;

pub(crate) struct PositionedEnvelopeSeed<'a, 'r, 'c, 's>(
    pub(crate) &'a ContentDecodeContext<'r, 'c, 's>,
);
impl<'de> DeserializeSeed<'de> for PositionedEnvelopeSeed<'_, '_, '_, '_> {
    type Value = (PatchStreamPosition, CanonicalEnvelopeWire<'de>);
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for PositionedEnvelopeSeed<'_, '_, '_, '_> {
    type Value = (PatchStreamPosition, CanonicalEnvelopeWire<'de>);
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("positioned canonical envelope")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut position = None;
        let mut canonical = None;
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
                _ => return Err(M::Error::unknown_field(key, &["position", "canonical"])),
            }
        }
        Ok((
            position.ok_or_else(|| M::Error::missing_field("position"))?,
            canonical.ok_or_else(|| M::Error::missing_field("canonical"))?,
        ))
    }
}
