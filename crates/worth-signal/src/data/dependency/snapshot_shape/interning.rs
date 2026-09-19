//! Append-only shape interning under an explicit evaluation work owner.
use super::{DependencySnapshotShape, DependencySnapshotShapeStore, SnapshotShapeHandle};
use crate::data::error::SignalError;
use crate::logic::evaluation::EvaluationWork;

use super::insertion::PreparedShapeInsertion;

impl DependencySnapshotShapeStore {
    pub(in crate::data::dependency) fn prepare_intern_with_work(
        &mut self,
        shape: DependencySnapshotShape,
        work: &mut EvaluationWork<'_>,
    ) -> Result<PreparedShapeInsertion, SignalError> {
        if shape.as_slice().is_empty() {
            return Ok(PreparedShapeInsertion::existing(SnapshotShapeHandle::EMPTY));
        }
        match work {
            EvaluationWork::Ordinary => self.rebuild_interner_if_needed(),
            EvaluationWork::Conditional(_) if !self.retained_interner_is_complete() => {
                return Err(SignalError::SnapshotIndexUnavailable)
            }
            EvaluationWork::Conditional(_) => {}
        }
        work.reserve(Some(shape.as_slice().len()))?;
        let mut query = Some(16usize);
        for key in shape.as_slice() {
            query = query
                .and_then(|n| n.checked_add(16))
                .and_then(|n| n.checked_add(key.scope.as_ref().map_or(0, |s| s.partition.0.len())))
                .and_then(|n| {
                    n.checked_add(
                        key.scope
                            .as_ref()
                            .and_then(|s| s.detail.as_ref())
                            .map_or(0, String::len),
                    )
                });
        }
        let comparison = query.and_then(|n| n.checked_mul(2));
        let steps = self.interner.lookup_steps();
        work.reserve(comparison.and_then(|n| n.checked_mul(steps)))?;
        if let Some(handle) = self.interner.get(&shape).copied() {
            return Ok(PreparedShapeInsertion::existing(handle));
        }
        // The closed owner only appends shapes: a missing key cannot be a
        // retired base key. Insertion reads its query paths and copies fixed
        // Arc-backed keys/handles; it never clones other shapes' payloads.
        work.reserve(
            comparison
                .and_then(|n| n.checked_mul(steps))
                .and_then(|n| n.checked_mul(3))
                .and_then(|n| n.checked_add(steps.checked_mul(16)?)),
        )?;
        let vector_work = match self.shapes.exclusive_capacity() {
            Some(capacity) if capacity == self.shapes.len() => self
                .shapes
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(std::mem::size_of::<DependencySnapshotShape>())),
            Some(_) => Some(1),
            None => self.shapes.lookup_steps().checked_mul(16).and_then(|n| {
                n.checked_add(32 * std::mem::size_of::<DependencySnapshotShape>() + 64)
            }),
        };
        work.reserve(vector_work)?;
        let next = self
            .shapes
            .len()
            .checked_add(1)
            .filter(|n| *n <= u32::MAX as usize)
            .ok_or_else(|| {
                SignalError::invalid_input("snapshot shape handle capacity exhausted")
            })?;
        let handle = SnapshotShapeHandle::from_index(next);
        Ok(PreparedShapeInsertion::new(
            shape,
            handle,
            self.shapes.len(),
        ))
    }
    pub(crate) fn intern_with_work(
        &mut self,
        shape: DependencySnapshotShape,
        work: &mut EvaluationWork<'_>,
    ) -> Result<SnapshotShapeHandle, SignalError> {
        let insertion = self.prepare_intern_with_work(shape, work)?;
        Ok(insertion.publish(self))
    }
}

#[cfg(test)]
mod tests;
