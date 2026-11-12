use super::RelationalCheckpointRoot;
use crate::execution::RelationalExecutionDenialCause as Cause;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
struct Backing {
    roots: Vec<RelationalCheckpointRoot>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl Clone for Backing {
    fn clone(&self) -> Self {
        // Called only by the explicitly ordinary mutation door.
        Self {
            roots: self.roots.clone(),
            _memory: None,
        }
    }
}
#[derive(Debug, Clone, Default)]
pub(crate) struct CheckpointRoots(Option<Arc<Backing>>);
impl std::ops::Deref for CheckpointRoots {
    type Target = [RelationalCheckpointRoot];
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().map_or(&[], |owner| owner.roots.as_slice())
    }
}
impl CheckpointRoots {
    pub(crate) fn from_ordinary(roots: Vec<RelationalCheckpointRoot>) -> Self {
        if roots.is_empty() {
            return Self::default();
        }
        Self(Some(Arc::new(Backing {
            roots,
            _memory: None,
        })))
    }
    pub(crate) fn push_ordinary(&mut self, root: RelationalCheckpointRoot) {
        let owner = self.0.get_or_insert_with(|| {
            Arc::new(Backing {
                roots: Vec::new(),
                _memory: None,
            })
        });
        Arc::make_mut(owner).roots.push(root);
    }
}
impl<'a> IntoIterator for &'a CheckpointRoots {
    type Item = &'a RelationalCheckpointRoot;
    type IntoIter = std::slice::Iter<'a, RelationalCheckpointRoot>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl IntoIterator for CheckpointRoots {
    type Item = RelationalCheckpointRoot;
    type IntoIter = std::vec::IntoIter<RelationalCheckpointRoot>;
    fn into_iter(self) -> Self::IntoIter {
        // Ordinary consuming image reconstruction; no controlled-copy claim.
        self.iter().cloned().collect::<Vec<_>>().into_iter()
    }
}
impl PartialEq for CheckpointRoots {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl Eq for CheckpointRoots {}
impl AsRef<[RelationalCheckpointRoot]> for CheckpointRoots {
    fn as_ref(&self) -> &[RelationalCheckpointRoot] {
        self
    }
}
impl Serialize for CheckpointRoots {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_ref().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CheckpointRoots {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<RelationalCheckpointRoot>::deserialize(deserializer).map(Self::from_ordinary)
    }
}
pub(crate) struct PreparedCheckpointRoots {
    roots: Vec<RelationalCheckpointRoot>,
    expected: usize,
    memory: Option<ExecutionMemoryReservation>,
}
impl PreparedCheckpointRoots {
    pub(crate) fn prepare(count: usize, request: &ExecutionRequest<'_, '_>) -> Result<Self, Cause> {
        let memory = if count == 0 {
            None
        } else {
            let bytes = super::root::control_bytes::<Backing>()?
                .checked_add(super::root::array_bytes::<RelationalCheckpointRoot>(count)?)
                .ok_or(Cause::ChargedBytesOverflow)?;
            request
                .reserve_retained_memory(bytes)
                .map_err(Cause::from)?
        };
        Ok(Self {
            roots: Vec::with_capacity(count),
            expected: count,
            memory,
        })
    }
    pub(crate) fn push(&mut self, root: RelationalCheckpointRoot) -> Result<(), Cause> {
        if self.roots.len() == self.expected {
            return Err(Cause::ChargedBytesOverflow);
        }
        self.roots.push(root);
        Ok(())
    }
    pub(crate) fn finish(self) -> Result<CheckpointRoots, Cause> {
        if self.roots.len() != self.expected {
            return Err(Cause::ChargedBytesOverflow);
        }
        if self.expected == 0 {
            return Ok(CheckpointRoots::default());
        }
        Ok(CheckpointRoots(Some(Arc::new(Backing {
            roots: self.roots,
            _memory: self.memory,
        }))))
    }
}
