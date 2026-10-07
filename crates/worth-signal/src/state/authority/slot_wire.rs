//! Field identity selects current node images or historical in-memory entries.
//! Decode directly so binary field readers retain the carrier's readability mode.
use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::data::node::{CheckpointNodeImage, NodeEntry};

use super::SignalCheckpointSlot;

impl<'de> Deserialize<'de> for SignalCheckpointSlot {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_struct(
            "SignalCheckpointSlot",
            &["node", "generation", "retired"],
            SlotVisitor,
        )
    }
}

struct SlotVisitor;

impl<'de> Visitor<'de> for SlotVisitor {
    type Value = SignalCheckpointSlot;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a checkpoint slot with a node image or legacy entry")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<Self::Value, S::Error> {
        let node = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let generation = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let retired = sequence
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;
        if sequence.next_element::<de::IgnoredAny>()?.is_some() {
            return Err(de::Error::invalid_length(4, &self));
        }
        Ok(SignalCheckpointSlot {
            node,
            generation,
            retired,
        })
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut node = None;
        let mut generation = None;
        let mut retired = None;
        while let Some(field) = map.next_key::<String>()? {
            match field.as_str() {
                "node" | "entry" => {
                    if node.is_some() {
                        return Err(de::Error::duplicate_field("node"));
                    }
                    node = Some(if field == "node" {
                        map.next_value::<Option<CheckpointNodeImage>>()?
                    } else {
                        map.next_value::<Option<NodeEntry>>()?
                            .map(|entry| entry.to_checkpoint_image())
                    });
                }
                "generation" => {
                    if generation.is_some() {
                        return Err(de::Error::duplicate_field("generation"));
                    }
                    generation = Some(map.next_value()?);
                }
                "retired" => {
                    if retired.is_some() {
                        return Err(de::Error::duplicate_field("retired"));
                    }
                    retired = Some(map.next_value()?);
                }
                _ => {
                    map.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        Ok(SignalCheckpointSlot {
            node: node.unwrap_or(None),
            generation: generation.ok_or_else(|| de::Error::missing_field("generation"))?,
            retired: retired.ok_or_else(|| de::Error::missing_field("retired"))?,
        })
    }
}
