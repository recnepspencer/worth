//! Required owner-issued definition prefix, independent of derived generations.
use crate::indexes::data::DerivedIndexDefinition;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug)]
struct Backing {
    definitions: Vec<DerivedIndexDefinition>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Debug, Clone)]
pub(crate) struct CanonicalIndexDefinitionDelta {
    pub(crate) base_next_index_id: u64,
    pub(crate) next_index_id: u64,
    backing: Option<Arc<Backing>>,
}

#[derive(Serialize)]
struct WireRef<'a> {
    base_next_index_id: u64,
    next_index_id: u64,
    definitions: &'a [DerivedIndexDefinition],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    base_next_index_id: u64,
    next_index_id: u64,
    definitions: Vec<DerivedIndexDefinition>,
}

impl CanonicalIndexDefinitionDelta {
    /// Attach only before the freshly deserialized private backing can alias.
    /// Alias/slot refusal returns the exact original unconsumed reservation.
    pub(crate) fn retain_decoded_memory(
        &mut self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), Option<worth_execution::ExecutionMemoryReservation>> {
        let Some(backing) = self.backing.as_mut() else {
            return if memory.is_none() {
                Ok(())
            } else {
                Err(memory)
            };
        };
        let Some(backing) = Arc::get_mut(backing) else {
            return Err(memory);
        };
        if backing._memory.is_some() {
            return Err(memory);
        }
        backing._memory = memory;
        Ok(())
    }

    pub(crate) fn root_bytes() -> Option<u64> {
        Some(
            std::alloc::Layout::new::<(usize, usize)>()
                .extend(std::alloc::Layout::new::<Backing>())
                .ok()?
                .0
                .pad_to_align()
                .size() as u64,
        )
    }
    pub(crate) fn prepared(
        base: u64,
        next: u64,
        definitions: Vec<DerivedIndexDefinition>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            base_next_index_id: base,
            next_index_id: next,
            backing: (!definitions.is_empty()).then(|| {
                Arc::new(Backing {
                    definitions,
                    _memory: memory,
                })
            }),
        }
    }
    pub(crate) fn unpaid_owned_bytes(&self) -> u64 {
        self.backing.as_ref().map_or(0, |backing| {
            if backing._memory.is_some() {
                return 0;
            }
            Self::root_bytes()
                .unwrap_or(u64::MAX)
                .saturating_add(
                    (backing.definitions.capacity() as u64)
                        .saturating_mul(std::mem::size_of::<DerivedIndexDefinition>() as u64),
                )
                .saturating_add(
                    backing
                        .definitions
                        .iter()
                        .map(DerivedIndexDefinition::owned_allocation_capacity_bytes)
                        .fold(0u64, u64::saturating_add),
                )
        })
    }
    pub(crate) fn definitions(&self) -> &[DerivedIndexDefinition] {
        self.backing
            .as_ref()
            .map_or(&[], |backing| &backing.definitions)
    }
    pub(crate) fn shape_valid(&self) -> bool {
        self.base_next_index_id > 0
            && self.next_index_id.checked_sub(self.base_next_index_id)
                == Some(self.definitions().len() as u64)
            && self
                .definitions()
                .iter()
                .enumerate()
                .all(|(offset, definition)| {
                    self.base_next_index_id.checked_add(offset as u64)
                        == Some(definition.index_id.0)
                })
    }
}
impl PartialEq for CanonicalIndexDefinitionDelta {
    fn eq(&self, other: &Self) -> bool {
        self.base_next_index_id == other.base_next_index_id
            && self.next_index_id == other.next_index_id
            && self.definitions() == other.definitions()
    }
}
impl Eq for CanonicalIndexDefinitionDelta {}
impl Serialize for CanonicalIndexDefinitionDelta {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        WireRef {
            base_next_index_id: self.base_next_index_id,
            next_index_id: self.next_index_id,
            definitions: self.definitions(),
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CanonicalIndexDefinitionDelta {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = Wire::deserialize(deserializer)?;
        let delta = Self::prepared(
            wire.base_next_index_id,
            wire.next_index_id,
            wire.definitions,
            None,
        );
        if !delta.shape_valid() {
            return Err(serde::de::Error::custom("invalid index definition delta"));
        }
        Ok(delta)
    }
}
