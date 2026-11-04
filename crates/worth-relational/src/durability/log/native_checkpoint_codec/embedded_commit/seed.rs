//! Embedded checkpoint envelopes use the same controlled Native content ingress.
use super::{EmbeddedCommitWire, NativeCommitWire};
use crate::content::decoding::ContentDecodeContext;
use crate::history::data::PositionedEnvelopeSeed;
use serde::de::{DeserializeSeed, EnumAccess, Error, VariantAccess, Visitor};
use serde::Deserializer;

pub(super) struct EmbeddedCommitSeed<'a, 'r, 'c, 's>(
    pub(super) &'a ContentDecodeContext<'r, 'c, 's>,
);
impl<'de> DeserializeSeed<'de> for EmbeddedCommitSeed<'_, '_, '_, '_> {
    type Value = EmbeddedCommitWire<'de>;
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_enum("EmbeddedCommitWire", &["Native", "Journal"], self)
    }
}
impl<'de> Visitor<'de> for EmbeddedCommitSeed<'_, '_, '_, '_> {
    type Value = EmbeddedCommitWire<'de>;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("embedded Native or journal envelope")
    }
    fn visit_enum<A: EnumAccess<'de>>(self, input: A) -> Result<Self::Value, A::Error> {
        let (variant, value) = input.variant::<&str>()?;
        match variant {
            "Native" => {
                let (position, canonical) =
                    value.newtype_variant_seed(PositionedEnvelopeSeed(self.0))?;
                Ok(EmbeddedCommitWire::Native(NativeCommitWire {
                    position,
                    canonical,
                }))
            }
            "Journal" => Ok(EmbeddedCommitWire::Journal(value.newtype_variant()?)),
            _ => Err(A::Error::unknown_variant(variant, &["Native", "Journal"])),
        }
    }
}
