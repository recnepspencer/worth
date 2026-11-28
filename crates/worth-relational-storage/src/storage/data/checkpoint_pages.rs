//! Intrinsic custody of the actual RelationalCheckpointPage directory backing.
use super::checkpoint_page::RelationalCheckpointPage;
use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use crate::storage::substrate::storage_allocation::StorageAllocation;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::alloc::Layout;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[derive(Debug, Clone, Default)]
pub struct CheckpointPages(Option<StorageAllocation<Vec<RelationalCheckpointPage>>>);
pub struct CheckpointPagesBuilder {
    rows: Vec<RelationalCheckpointPage>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CheckpointPages {
    pub fn shares_backing(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(left), Some(right)) => left.shares_backing(right),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        }
    }

    pub fn from_ordinary(rows: Vec<RelationalCheckpointPage>) -> Self {
        if rows.is_empty() {
            Self::default()
        } else {
            Self(Some(StorageAllocation::new(rows)))
        }
    }
    pub fn ordinary_owned_copy(&self) -> Vec<RelationalCheckpointPage> {
        self.rows().to_vec()
    }
    pub fn allocate_for_request(
        capacity: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CheckpointPagesBuilder, Cause> {
        crate::execution::request_custody::check_request(request)?;
        if capacity == 0 {
            return Ok(CheckpointPagesBuilder {
                rows: Vec::new(),
                _memory: None,
            });
        }
        let control = StorageAllocation::<Vec<RelationalCheckpointPage>>::layout()
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .size();
        let rows = Layout::array::<RelationalCheckpointPage>(capacity)
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .size();
        let bytes = control
            .checked_add(rows)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        Ok(CheckpointPagesBuilder {
            rows: Vec::with_capacity(capacity),
            _memory: memory,
        })
    }
    fn rows(&self) -> &[RelationalCheckpointPage] {
        self.0.as_ref().map_or(&[], |b| b.as_slice())
    }
}
impl CheckpointPagesBuilder {
    fn room(&self) -> Result<(), Cause> {
        if self.rows.len() == self.rows.capacity() {
            Err(Cause::ChargedBytesOverflow)
        } else {
            Ok(())
        }
    }
    pub fn seal(self) -> CheckpointPages {
        if self.rows.is_empty() {
            return CheckpointPages::default();
        }
        CheckpointPages(Some(StorageAllocation::from_reserved(
            self.rows,
            self._memory,
        )))
    }
}
impl std::ops::Deref for CheckpointPages {
    type Target = [RelationalCheckpointPage];
    fn deref(&self) -> &Self::Target {
        self.rows()
    }
}
impl PartialEq for CheckpointPages {
    fn eq(&self, other: &Self) -> bool {
        self.rows() == other.rows()
    }
}
impl Eq for CheckpointPages {}
impl Serialize for CheckpointPages {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.rows().serialize(s)
    }
}
impl<'de> Deserialize<'de> for CheckpointPages {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Vec::<RelationalCheckpointPage>::deserialize(d).map(Self::from_ordinary)
    }
}
impl<'a> IntoIterator for &'a CheckpointPages {
    type Item = &'a RelationalCheckpointPage;
    type IntoIter = std::slice::Iter<'a, RelationalCheckpointPage>;
    fn into_iter(self) -> Self::IntoIter {
        self.rows().iter()
    }
}

impl CheckpointPagesBuilder {
    pub fn push(&mut self, row: RelationalCheckpointPage) -> Result<(), Cause> {
        self.room()?;
        self.rows.push(row);
        Ok(())
    }
}
