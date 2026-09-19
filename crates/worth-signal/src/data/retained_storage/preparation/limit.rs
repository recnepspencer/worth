use super::RetainedStoragePreparation;
use std::ops::{Deref, DerefMut};

/// A borrowed ceiling over the same monotonic work counter. Drop restores only
/// the outer ceiling, including on unwind; consumed visits are never refunded.
pub(crate) struct RetainedStoragePreparationLimit<'a> {
    work: &'a mut RetainedStoragePreparation,
    outer_maximum: usize,
    outer_limited: bool,
}

impl<'a> RetainedStoragePreparationLimit<'a> {
    pub(super) fn new(work: &'a mut RetainedStoragePreparation, maximum_additional: usize) -> Self {
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

impl Deref for RetainedStoragePreparationLimit<'_> {
    type Target = RetainedStoragePreparation;
    fn deref(&self) -> &Self::Target {
        self.work
    }
}

impl DerefMut for RetainedStoragePreparationLimit<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.work
    }
}

impl Drop for RetainedStoragePreparationLimit<'_> {
    fn drop(&mut self) {
        self.work.maximum_visits = self.outer_maximum;
    }
}

#[cfg(test)]
mod tests;
