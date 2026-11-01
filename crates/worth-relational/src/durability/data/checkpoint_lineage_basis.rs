//! Published lineage totals retain their actual copied ID roster through every alias.
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::lineage::data::LineageCheckpointDigestBasis;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
struct Backing {
    basis: LineageCheckpointDigestBasis,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone)]
pub(crate) struct CheckpointLineageBasis(Arc<Backing>);
impl CheckpointLineageBasis {
    pub(crate) fn from_original(
        commits: usize,
        events: usize,
        decisions: usize,
        id_count: usize,
        ids: impl Iterator<Item = u64>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Cause> {
        let control = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Backing>())
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .0
            .pad_to_align()
            .size();
        let roster = Layout::array::<u64>(id_count)
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .size();
        let bytes = control
            .checked_add(roster)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        let mut canonical_ids = Vec::with_capacity(id_count);
        for id in ids {
            crate::execution::check_request(request)?;
            if canonical_ids.len() == id_count {
                return Err(Cause::ChargedBytesOverflow);
            }
            canonical_ids.push(id);
        }
        if canonical_ids.len() != id_count {
            return Err(Cause::ChargedBytesOverflow);
        }
        Ok(Self(Arc::new(Backing {
            basis: LineageCheckpointDigestBasis::new(commits, canonical_ids, events, decisions),
            _memory: memory,
        })))
    }
    pub(crate) fn from_ordinary(basis: LineageCheckpointDigestBasis) -> Self {
        Self(Arc::new(Backing {
            basis,
            _memory: None,
        }))
    }
    /// This independent public DTO copy has the existing ordinary allocation contract.
    pub(crate) fn ordinary_owned_copy(&self) -> LineageCheckpointDigestBasis {
        self.0.basis.clone()
    }
}
impl std::ops::Deref for CheckpointLineageBasis {
    type Target = LineageCheckpointDigestBasis;
    fn deref(&self) -> &Self::Target {
        &self.0.basis
    }
}
impl PartialEq for CheckpointLineageBasis {
    fn eq(&self, other: &Self) -> bool {
        self.0.basis == other.0.basis
    }
}
impl Eq for CheckpointLineageBasis {}
impl Serialize for CheckpointLineageBasis {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.basis.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CheckpointLineageBasis {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        LineageCheckpointDigestBasis::deserialize(deserializer).map(Self::from_ordinary)
    }
}
