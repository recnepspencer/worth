//! Intrinsic custody of the actual RelationalCheckpointPage directory backing.
use super::RelationalCheckpointPage;
use crate::execution::RelationalExecutionDenialCause as Cause;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[derive(Debug)]
struct Backing {
    rows: Vec<RelationalCheckpointPage>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct CheckpointPages(Option<Arc<Backing>>);
pub(crate) struct CheckpointPagesBuilder {
    rows: Vec<RelationalCheckpointPage>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CheckpointPages {
    pub(crate) fn from_ordinary(rows: Vec<RelationalCheckpointPage>) -> Self {
        if rows.is_empty() {
            Self::default()
        } else {
            Self(Some(Arc::new(Backing {
                rows,
                _memory: None,
            })))
        }
    }
    pub(crate) fn ordinary_owned_copy(&self) -> Vec<RelationalCheckpointPage> {
        self.rows().to_vec()
    }
    pub(crate) fn allocate_for_request(
        capacity: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CheckpointPagesBuilder, Cause> {
        crate::execution::check_request(request)?;
        if capacity == 0 {
            return Ok(CheckpointPagesBuilder {
                rows: Vec::new(),
                _memory: None,
            });
        }
        let control = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Backing>())
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .0
            .pad_to_align()
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
        self.0.as_ref().map_or(&[], |b| b.rows.as_slice())
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
    pub(crate) fn seal(self) -> CheckpointPages {
        if self.rows.is_empty() {
            return CheckpointPages::default();
        }
        CheckpointPages(Some(Arc::new(Backing {
            rows: self.rows,
            _memory: self._memory,
        })))
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
    pub(crate) fn push(&mut self, row: RelationalCheckpointPage) -> Result<(), Cause> {
        self.room()?;
        self.rows.push(row);
        Ok(())
    }
}
