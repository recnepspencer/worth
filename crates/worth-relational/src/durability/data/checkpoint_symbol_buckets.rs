//! Intrinsic custody of the actual RelationalCheckpointSymbolBucket directory backing.
use super::RelationalCheckpointSymbolBucket;
use crate::execution::RelationalExecutionDenialCause as Cause;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
#[derive(Debug)]
struct Backing {
    rows: Vec<RelationalCheckpointSymbolBucket>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct CheckpointSymbolBuckets(Option<Arc<Backing>>);
pub(crate) struct CheckpointSymbolBucketsBuilder {
    rows: Vec<RelationalCheckpointSymbolBucket>,
    bytes: u64,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CheckpointSymbolBuckets {
    pub(crate) fn shares_backing(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        }
    }

    pub(crate) fn from_ordinary(rows: Vec<RelationalCheckpointSymbolBucket>) -> Self {
        if rows.is_empty() {
            Self::default()
        } else {
            Self(Some(Arc::new(Backing {
                rows,
                _memory: None,
            })))
        }
    }
    pub(crate) fn ordinary_owned_copy(&self) -> Vec<RelationalCheckpointSymbolBucket> {
        self.rows().to_vec()
    }
    pub(crate) fn allocate_for_request(
        capacity: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CheckpointSymbolBucketsBuilder, Cause> {
        crate::execution::check_request(request)?;
        if capacity == 0 {
            return Ok(CheckpointSymbolBucketsBuilder {
                rows: Vec::new(),
                bytes: 0,
                _memory: None,
            });
        }
        let control = Layout::new::<[usize; 2]>()
            .extend(Layout::new::<Backing>())
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .0
            .pad_to_align()
            .size();
        let rows = Layout::array::<RelationalCheckpointSymbolBucket>(capacity)
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .size();
        let bytes = control
            .checked_add(rows)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        Ok(CheckpointSymbolBucketsBuilder {
            rows: Vec::with_capacity(capacity),
            bytes,
            _memory: memory,
        })
    }
    fn rows(&self) -> &[RelationalCheckpointSymbolBucket] {
        self.0.as_ref().map_or(&[], |b| b.rows.as_slice())
    }
}
impl CheckpointSymbolBucketsBuilder {
    fn admit_nested(
        &mut self,
        bytes: usize,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        crate::execution::check_request(request)?;
        let total = self
            .bytes
            .checked_add(u64::try_from(bytes).map_err(|_| Cause::ChargedBytesOverflow)?)
            .ok_or(Cause::ChargedBytesOverflow)?;
        if let Some(memory) = &mut self._memory {
            memory.resize(total).map_err(|cause| {
                crate::execution::denial::lease_denial(
                    worth_execution::LeaseDenial::MemoryExhausted(cause),
                )
            })?;
        } else {
            self._memory = request
                .reserve_retained_memory(total)
                .map_err(Cause::from)?;
        }
        self.bytes = total;
        Ok(())
    }
    fn room(&self) -> Result<(), Cause> {
        if self.rows.len() == self.rows.capacity() {
            Err(Cause::ChargedBytesOverflow)
        } else {
            Ok(())
        }
    }
    pub(crate) fn seal(self) -> CheckpointSymbolBuckets {
        if self.rows.is_empty() {
            return CheckpointSymbolBuckets::default();
        }
        CheckpointSymbolBuckets(Some(Arc::new(Backing {
            rows: self.rows,
            _memory: self._memory,
        })))
    }
}
impl std::ops::Deref for CheckpointSymbolBuckets {
    type Target = [RelationalCheckpointSymbolBucket];
    fn deref(&self) -> &Self::Target {
        self.rows()
    }
}
impl PartialEq for CheckpointSymbolBuckets {
    fn eq(&self, other: &Self) -> bool {
        self.rows() == other.rows()
    }
}
impl Eq for CheckpointSymbolBuckets {}
impl Serialize for CheckpointSymbolBuckets {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.rows().serialize(s)
    }
}
impl<'de> Deserialize<'de> for CheckpointSymbolBuckets {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Vec::<RelationalCheckpointSymbolBucket>::deserialize(d).map(Self::from_ordinary)
    }
}
impl<'a> IntoIterator for &'a CheckpointSymbolBuckets {
    type Item = &'a RelationalCheckpointSymbolBucket;
    type IntoIter = std::slice::Iter<'a, RelationalCheckpointSymbolBucket>;
    fn into_iter(self) -> Self::IntoIter {
        self.rows().iter()
    }
}

impl CheckpointSymbolBucketsBuilder {
    pub(crate) fn push(
        &mut self,
        bucket: u8,
        pages: &[super::super::medium::RelationalDurableMemberLocator],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Cause> {
        self.room()?;
        let bytes =
            Layout::array::<super::super::medium::RelationalDurableMemberLocator>(pages.len())
                .map_err(|_| Cause::ChargedBytesOverflow)?
                .size();
        self.admit_nested(bytes, request)?;
        self.rows.push(RelationalCheckpointSymbolBucket {
            bucket,
            pages: pages.to_vec(),
        });
        Ok(())
    }
}
