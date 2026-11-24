//! Context-free decoding carries descriptive System text and no live admission.
use super::{CanonicalCommitEnvelope, CanonicalEnvelopeWire};
use crate::symbols::data::SymbolTableSnapshot;
use serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for CanonicalCommitEnvelope {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // The flat typed wire avoids Serde flatten's unsupported u128 buffer.
        // Managed text still refuses in SymbolTableSnapshot's own decoder.
        let mut wire = CanonicalEnvelopeWire::<SymbolTableSnapshot>::deserialize(deserializer)?;
        let symbols = wire.take_descriptive_symbols();
        wire.into_envelope(symbols)
            .map_err(serde::de::Error::custom)
    }
}
