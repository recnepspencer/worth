mod fork_growth;
mod mutation_work;
mod persistent_fork;
mod replacement;
mod replacement_charge;
mod staging_charge;
use crate::data::retained_storage::RetainedStorageBacking;
use std::sync::Arc;

mod capacity_edit;
#[cfg(test)]
mod capacity_edit_tests;
#[cfg(test)]
mod charge_update_tests;
mod charge_updates;
mod fork_page;
mod iteration;
mod retained_charge;
#[cfg(test)]
mod retained_charge_tests;
mod serialization;
#[cfg(test)]
mod tests;
mod traits;

use self::fork_page::ForkPage;
use self::iteration::PersistentVectorIter;
use crate::data::retained_storage::RetainedStorageCharge;
pub(crate) use capacity_edit::{
    RetainedVectorCapacityDenial, RetainedVectorCapacityOutcome, RetainedVectorStagingDenial,
};
pub(crate) use charge_updates::{RetainedVectorMutationDenial, RetainedVectorMutationOutcome};

const DEFAULT_PAGE_LEN: usize = 32;

enum PersistentVectorStorage<T> {
    Exclusive(Vec<T>),
    ForkShared {
        base: Arc<RetainedStorageBacking<Vec<T>>>,
        changed_pages: im::OrdMap<usize, Arc<ForkPage<T>>>,
        len: usize,
    },
}

/// A sequence that keeps the non-forking lane flat and detaches bounded pages
/// only after an exact owner-cell fork.
pub(crate) struct PersistentVector<T: Clone, const PAGE_LEN: usize = DEFAULT_PAGE_LEN> {
    storage: PersistentVectorStorage<T>,
    retained_charge: Option<RetainedStorageCharge>,
}

impl<T: Clone, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    pub(crate) fn new() -> Self {
        debug_assert!(PAGE_LEN != 0);
        Self {
            storage: PersistentVectorStorage::Exclusive(Vec::new()),
            retained_charge: Some(RetainedStorageCharge::ZERO),
        }
    }

    #[inline(always)]
    pub(crate) fn len(&self) -> usize {
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => values.len(),
            PersistentVectorStorage::ForkShared { len, .. } => *len,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn reserve_exclusive(&mut self, additional: usize) {
        if let PersistentVectorStorage::Exclusive(values) = &mut self.storage {
            let previous = self.retained_charge.take();
            let old_capacity = values.capacity();
            values.reserve(additional);
            self.retained_charge = previous.and_then(|charge| {
                charge
                    .checked_sub(RetainedStorageCharge::capacity::<T>(old_capacity).ok()?)
                    .ok()?
                    .checked_add(RetainedStorageCharge::capacity::<T>(values.capacity()).ok()?)
                    .ok()
            });
        }
    }

    pub(crate) fn exclusive_capacity(&self) -> Option<usize> {
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => Some(values.capacity()),
            PersistentVectorStorage::ForkShared { .. } => None,
        }
    }

    /// Structural lookup bound for fixed usize page keys; payloads are borrowed.
    pub(crate) fn lookup_steps(&self) -> usize {
        match &self.storage {
            PersistentVectorStorage::Exclusive(_) => 1,
            PersistentVectorStorage::ForkShared { changed_pages, .. } => {
                crate::data::retained_storage::ordered_lookup_steps(changed_pages.len()) + 2
            }
        }
    }

    #[inline(always)]
    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => values.get(index),
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                len,
            } => {
                if index >= *len {
                    return None;
                }
                let page_index = index / PAGE_LEN;
                changed_pages.get(&page_index).map_or_else(
                    || base.get(index),
                    |page| page.get(base, index, index % PAGE_LEN),
                )
            }
        }
    }

    #[inline(always)]
    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentVectorStorage::Exclusive(values) => values.get_mut(index),
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                len,
            } => {
                if index >= *len {
                    return None;
                }
                let page_index = index / PAGE_LEN;
                install_changed_page::<T, PAGE_LEN>(base, changed_pages, page_index);
                Arc::make_mut(
                    changed_pages
                        .get_mut(&page_index)
                        .expect("changed page must be installed"),
                )
                .get_mut(base, index, index % PAGE_LEN)
            }
        }
    }

    pub(crate) fn last(&self) -> Option<&T> {
        self.len().checked_sub(1).and_then(|index| self.get(index))
    }

    #[inline(always)]
    pub(crate) fn push_back(&mut self, value: T) {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentVectorStorage::Exclusive(values) => values.push(value),
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                len,
            } => {
                let page_index = *len / PAGE_LEN;
                install_changed_page::<T, PAGE_LEN>(base, changed_pages, page_index);
                Arc::make_mut(
                    changed_pages
                        .get_mut(&page_index)
                        .expect("changed page must be installed"),
                )
                .push(value, *len % PAGE_LEN);
                *len += 1;
            }
        }
    }

    pub(crate) fn pop_back(&mut self) -> Option<T> {
        self.retained_charge = None;
        match &mut self.storage {
            PersistentVectorStorage::Exclusive(values) => values.pop(),
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                len,
            } => {
                let index = len.checked_sub(1)?;
                let page_index = index / PAGE_LEN;
                install_changed_page::<T, PAGE_LEN>(base, changed_pages, page_index);
                let (value, page_is_empty) = {
                    let page = Arc::make_mut(
                        changed_pages
                            .get_mut(&page_index)
                            .expect("changed page must be installed"),
                    );
                    let value = page.pop(base, index, index % PAGE_LEN);
                    (value, page.is_empty())
                };
                *len = index;
                if page_is_empty {
                    changed_pages.remove(&page_index);
                }
                value
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        self.storage = PersistentVectorStorage::Exclusive(Vec::new());
        self.retained_charge = Some(RetainedStorageCharge::ZERO);
    }

    pub(crate) fn insert(&mut self, index: usize, value: T) {
        self.make_exclusive().insert(index, value);
    }

    pub(crate) fn binary_search_by_key<B, F>(&self, key: &B, mut key_of: F) -> Result<usize, usize>
    where
        B: Ord,
        F: FnMut(&T) -> B,
    {
        let mut left = 0;
        let mut right = self.len();
        while left < right {
            let middle = left + (right - left) / 2;
            match key_of(&self[middle]).cmp(key) {
                std::cmp::Ordering::Less => left = middle + 1,
                std::cmp::Ordering::Equal => return Ok(middle),
                std::cmp::Ordering::Greater => right = middle,
            }
        }
        Err(left)
    }

    pub(crate) fn iter(&self) -> PersistentVectorIter<'_, T, PAGE_LEN> {
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => {
                PersistentVectorIter::Exclusive(values.iter())
            }
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                len,
            } => PersistentVectorIter::ForkShared {
                base,
                changed_pages,
                len: *len,
                next: 0,
            },
        }
    }

    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.make_exclusive().iter_mut()
    }

    pub(crate) fn extend<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = T>,
    {
        for value in values {
            self.push_back(value);
        }
    }

    pub(crate) fn operational_clone(&self) -> Self {
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => Self {
                storage: PersistentVectorStorage::Exclusive(values.clone()),
                retained_charge: None,
            },
            PersistentVectorStorage::ForkShared { .. } => self.iter().cloned().collect(),
        }
    }

    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        match (&self.storage, &other.storage) {
            (
                PersistentVectorStorage::ForkShared {
                    base: left_base,
                    changed_pages: left_pages,
                    ..
                },
                PersistentVectorStorage::ForkShared {
                    base: right_base,
                    changed_pages: right_pages,
                    ..
                },
            ) => Arc::ptr_eq(left_base, right_base) && left_pages.ptr_eq(right_pages),
            _ => false,
        }
    }

    #[cfg(test)]
    pub(crate) fn page_identities(&self) -> Vec<usize> {
        match &self.storage {
            PersistentVectorStorage::Exclusive(values) => values
                .chunks(PAGE_LEN)
                .map(|page| page.as_ptr() as usize)
                .collect(),
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                len,
            } => (0..len.div_ceil(PAGE_LEN))
                .map(|page_index| {
                    changed_pages.get(&page_index).map_or_else(
                        || base.as_ptr().wrapping_add(page_index * PAGE_LEN) as usize,
                        |page| Arc::as_ptr(page) as usize,
                    )
                })
                .collect(),
        }
    }

    fn make_exclusive(&mut self) -> &mut Vec<T> {
        self.retained_charge = None;
        if matches!(self.storage, PersistentVectorStorage::ForkShared { .. }) {
            let values = self.iter().cloned().collect();
            self.storage = PersistentVectorStorage::Exclusive(values);
        }
        match &mut self.storage {
            PersistentVectorStorage::Exclusive(values) => values,
            PersistentVectorStorage::ForkShared { .. } => unreachable!("storage was flattened"),
        }
    }
}

fn install_changed_page<T: Clone, const PAGE_LEN: usize>(
    base: &Arc<RetainedStorageBacking<Vec<T>>>,
    changed_pages: &mut im::OrdMap<usize, Arc<ForkPage<T>>>,
    page_index: usize,
) {
    if changed_pages.contains_key(&page_index) {
        return;
    }
    let start = page_index * PAGE_LEN;
    let base_len = base.len().saturating_sub(start).min(PAGE_LEN);
    changed_pages.insert(page_index, Arc::new(ForkPage::new(base_len)));
}

impl<T: Clone, const PAGE_LEN: usize> Clone for PersistentVector<T, PAGE_LEN> {
    fn clone(&self) -> Self {
        match &self.storage {
            PersistentVectorStorage::Exclusive(_) => self.operational_clone(),
            PersistentVectorStorage::ForkShared {
                base,
                changed_pages,
                len,
            } => Self {
                retained_charge: self.retained_charge,
                storage: PersistentVectorStorage::ForkShared {
                    base: Arc::clone(base),
                    changed_pages: changed_pages.clone(),
                    len: *len,
                },
            },
        }
    }
}
