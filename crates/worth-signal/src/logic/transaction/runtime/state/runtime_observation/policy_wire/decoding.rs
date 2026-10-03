//! Decode the exact v1 two-field sequence or the versioned three-field policy.
use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::{PolicyWire, WireTrigger, CURRENT_SCHEMA, LEGACY_SCHEMA};

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<PolicyWire, D::Error> {
    deserializer.deserialize_struct(
        "ObservationPolicy",
        &["schema_version", "trigger", "delivery_mode"],
        PolicyVisitor,
    )
}

struct PolicyVisitor;

impl<'de> Visitor<'de> for PolicyVisitor {
    type Value = PolicyWire;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a legacy v1 or versioned v2 observation policy")
    }

    fn visit_map<M: MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
        // The derived map decoder enforces field uniqueness and rejects unknown fields.
        PolicyWire::deserialize(de::value::MapAccessDeserializer::new(map))
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<Self::Value, S::Error> {
        let first: String = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let (schema_version, trigger, delivery_index) = match first.as_str() {
            CURRENT_SCHEMA | LEGACY_SCHEMA => {
                let trigger = sequence
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                (first, trigger, 2)
            }
            "Touched" => (LEGACY_SCHEMA.to_owned(), WireTrigger::Touched, 1),
            "Recomputed" => (LEGACY_SCHEMA.to_owned(), WireTrigger::Recomputed, 1),
            "MeaningfulChange" => (LEGACY_SCHEMA.to_owned(), WireTrigger::MeaningfulChange, 1),
            _ => return Err(de::Error::custom("unsupported observation policy schema")),
        };
        let delivery_mode = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(delivery_index, &self))?;
        if sequence.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(delivery_index + 2, &self));
        }
        Ok(PolicyWire {
            schema_version,
            trigger,
            delivery_mode,
        })
    }
}
