use super::RetainedStoragePreparation;
use std::ops::{Deref, DerefMut};

/// A borrowed ceiling over the same monotonic work counter. Drop restores only
/// the outer ceiling, including on unwind; consumed visits are never refunded.
pub(crate) struct RetainedStoragePreparationLimit<'a, 'observer> {
    work: &'a mut RetainedStoragePreparation<'observer>,
    outer_maximum: usize,
    outer_limited: bool,
}

impl<'a, 'observer> RetainedStoragePreparationLimit<'a, 'observer> {
    pub(super) fn new(
        work: &'a mut RetainedStoragePreparation<'observer>,
        maximum_additional: usize,
    ) -> Self {
        let outer_maximum = work.maximum_visits;
        let remaining = outer_maximum - work.visits;
        work.maximum_visits = work.visits + remaining.min(maximum_additional);
        Self {
            work,
            outer_maximum,
            outer_limited: remaining <= maximum_additional,
        }
    }

    pub(crate) fn outer_limited(&self) -> bool {
        self.outer_limited
    }
}

impl<'a, 'observer> Deref for RetainedStoragePreparationLimit<'a, 'observer> {
    type Target = RetainedStoragePreparation<'observer>;
    fn deref(&self) -> &Self::Target {
        self.work
    }
}

impl DerefMut for RetainedStoragePreparationLimit<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.work
    }
}

impl Drop for RetainedStoragePreparationLimit<'_, '_> {
    fn drop(&mut self) {
        self.work.maximum_visits = self.outer_maximum;
    }
}

/// One short request-checkpoint view over the owner's existing visit counter.
/// It cannot outlive either the owner borrow or the actual request callback.
pub(crate) struct RetainedStoragePreparationCheckpoint<'work, 'checkpoint, 'observer> {
    pub(super) owner: &'work mut RetainedStoragePreparation<'observer>,
    pub(super) scoped: RetainedStoragePreparation<'checkpoint>,
}

impl<'work, 'checkpoint, 'observer> Deref
    for RetainedStoragePreparationCheckpoint<'work, 'checkpoint, 'observer>
{
    type Target = RetainedStoragePreparation<'checkpoint>;
    fn deref(&self) -> &Self::Target {
        &self.scoped
    }
}

impl DerefMut for RetainedStoragePreparationCheckpoint<'_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scoped
    }
}

impl Drop for RetainedStoragePreparationCheckpoint<'_, '_, '_> {
    fn drop(&mut self) {
        self.owner.visits = self.scoped.visits;
    }
}

#[cfg(test)]
mod tests;
