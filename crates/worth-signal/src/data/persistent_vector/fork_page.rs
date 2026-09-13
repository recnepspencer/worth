use std::sync::Arc;

use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

pub(crate) struct ForkPage<T> {
    base_len: usize,
    overrides: Vec<(usize, Arc<T>)>,
    appended: Vec<Arc<T>>,
}

impl<T> ForkPage<T> {
    pub(super) fn new(base_len: usize) -> Self {
        Self {
            base_len,
            overrides: Vec::new(),
            appended: Vec::new(),
        }
    }

    pub(super) fn get<'a>(
        &'a self,
        base: &'a [T],
        absolute_index: usize,
        page_offset: usize,
    ) -> Option<&'a T> {
        if page_offset < self.base_len {
            return match self.override_position(page_offset) {
                Ok(position) => Some(self.overrides[position].1.as_ref()),
                Err(_) => base.get(absolute_index),
            };
        }
        self.appended
            .get(page_offset - self.base_len)
            .map(Arc::as_ref)
    }

    pub(super) fn push(&mut self, value: T, page_offset: usize) {
        if page_offset < self.base_len {
            match self.override_position(page_offset) {
                Ok(position) => self.overrides[position].1 = Arc::new(value),
                Err(position) => self
                    .overrides
                    .insert(position, (page_offset, Arc::new(value))),
            }
            return;
        }
        let appended_index = page_offset - self.base_len;
        assert_eq!(
            appended_index,
            self.appended.len(),
            "persistent vector append must extend its logical tail"
        );
        self.appended.push(Arc::new(value));
    }

    pub(super) fn replace(&mut self, value: T, page_offset: usize) {
        if page_offset < self.base_len {
            self.push(value, page_offset);
        } else {
            self.appended[page_offset - self.base_len] = Arc::new(value);
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.overrides.is_empty() && self.appended.is_empty()
    }

    fn override_position(&self, page_offset: usize) -> Result<usize, usize> {
        self.overrides
            .binary_search_by_key(&page_offset, |(offset, _)| *offset)
    }
}

impl<T: Clone> ForkPage<T> {
    pub(super) fn get_mut<'a>(
        &'a mut self,
        base: &[T],
        absolute_index: usize,
        page_offset: usize,
    ) -> Option<&'a mut T> {
        if page_offset < self.base_len {
            let position = match self.override_position(page_offset) {
                Ok(position) => position,
                Err(position) => {
                    let value = base.get(absolute_index)?.clone();
                    self.overrides
                        .insert(position, (page_offset, Arc::new(value)));
                    position
                }
            };
            return Some(Arc::make_mut(&mut self.overrides[position].1));
        }
        self.appended
            .get_mut(page_offset - self.base_len)
            .map(Arc::make_mut)
    }

    pub(super) fn pop(
        &mut self,
        base: &[T],
        absolute_index: usize,
        page_offset: usize,
    ) -> Option<T> {
        if page_offset < self.base_len {
            return match self.override_position(page_offset) {
                Ok(position) => Some(into_owned(self.overrides.remove(position).1)),
                Err(_) => base.get(absolute_index).cloned(),
            };
        }
        let appended_index = page_offset - self.base_len;
        assert_eq!(
            appended_index + 1,
            self.appended.len(),
            "persistent vector pop must remove its logical tail"
        );
        self.appended.pop().map(into_owned)
    }
}

impl<T> Clone for ForkPage<T> {
    fn clone(&self) -> Self {
        Self {
            base_len: self.base_len,
            overrides: self.overrides.clone(),
            appended: self.appended.clone(),
        }
    }
}

impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for ForkPage<T> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.overrides
            .retained_heap_charge(work)?
            .checked_add(self.appended.retained_heap_charge(work)?)
    }
}

impl<T: RetainedStorageMeasurement> ForkPage<T> {
    pub(super) fn mutation_granule_charge(
        &self,
        offset: usize,
        work: &mut Preparation,
    ) -> Result<Charge, Denial> {
        let owned = if offset < self.base_len {
            self.override_position(offset)
                .ok()
                .map(|position| &self.overrides[position].1)
        } else {
            self.appended.get(offset - self.base_len)
        };
        let payload = owned.map_or(Ok(Charge::ZERO), |value| value.retained_heap_charge(work))?;
        self.mutation_structure_charge()?.checked_add(payload)
    }
}

impl<T> ForkPage<T> {
    pub(super) fn replacement_structure_bound(&self, offset: usize) -> Result<Charge, Denial> {
        let mut overrides = self.overrides.capacity();
        if offset < self.base_len && self.override_position(offset).is_err() {
            // Rust 1.94 Vec growth for these fixed (usize, Arc<T>) entries:
            // at least four initially, then doubling. A shared page clone can
            // reduce capacity to len before insertion, so cover that case too.
            overrides = overrides.max(4).max(
                self.overrides
                    .len()
                    .checked_mul(2)
                    .ok_or(Denial::ChargeOverflow)?,
            );
        }
        Charge::capacity::<(usize, Arc<T>)>(overrides)?
            .checked_add(Charge::capacity::<Arc<T>>(self.appended.capacity())?)
    }

    pub(super) fn mutation_structure_charge(&self) -> Result<Charge, Denial> {
        Charge::capacity::<(usize, Arc<T>)>(self.overrides.capacity())?
            .checked_add(Charge::capacity::<Arc<T>>(self.appended.capacity())?)
    }
}

fn into_owned<T: Clone>(value: Arc<T>) -> T {
    match Arc::try_unwrap(value) {
        Ok(value) => value,
        Err(shared) => shared.as_ref().clone(),
    }
}
