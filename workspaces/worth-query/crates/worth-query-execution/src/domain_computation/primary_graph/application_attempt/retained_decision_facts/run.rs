use super::{AdmittedFactKey, RetainedFact, Slot, Slots, StoreDenial};
use std::cell::{Cell, RefCell};
use worth_execution::ExecutionArrayBuilder;

pub(in crate::domain_computation::primary_graph) struct Run<T> {
    pub(in crate::domain_computation::primary_graph) slots: Slots<T>,
    pub(in crate::domain_computation::primary_graph) used: usize,
    cursor: Cell<usize>,
}

impl<T> Run<T> {
    pub(in crate::domain_computation::primary_graph) fn empty(
        count: usize,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        policy.check_live()?;
        let mut slots = ExecutionArrayBuilder::allocate(count, policy.policy())?;
        for _ in 0..count {
            policy.check_live()?;
            slots.push(RefCell::new(None))?;
        }
        Ok(Self {
            slots: slots.seal()?,
            used: 0,
            cursor: Cell::new(0),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn locate(
        &self,
        key: &AdmittedFactKey,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Option<&Slot<T>>, StoreDenial> {
        policy.check_live()?;
        if self.used == 0 {
            return Ok(None);
        }
        if self.slots[0]
            .borrow()
            .as_ref()
            .unwrap()
            .key
            .compare(key, policy)?
            .is_gt()
            || self.slots[self.used - 1]
                .borrow()
                .as_ref()
                .unwrap()
                .key
                .compare(key, policy)?
                .is_lt()
        {
            return Ok(None);
        }
        let mut low = 0;
        let mut high = self.used;
        while low < high {
            let middle = low + (high - low) / 2;
            match self.slots[middle]
                .borrow()
                .as_ref()
                .unwrap()
                .key
                .compare(key, policy)?
            {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Ok(Some(&self.slots[middle])),
            }
        }
        Ok(None)
    }
    pub(in crate::domain_computation::primary_graph) fn head(&self) -> Option<&Slot<T>> {
        (self.cursor.get() < self.used).then(|| &self.slots[self.cursor.get()])
    }
    pub(in crate::domain_computation::primary_graph) fn take_head(&self) -> RetainedFact<T> {
        let index = self.cursor.get();
        self.cursor.set(index + 1);
        self.slots[index].borrow_mut().take().unwrap()
    }
    pub(in crate::domain_computation::primary_graph) fn merge(
        left: Self,
        right: Self,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        let count = left
            .used
            .checked_add(right.used)
            .ok_or(StoreDenial::Representability)?;
        // Both old arrays/tickets stay live while replacement admission happens.
        policy.check_live()?;
        let mut merged = ExecutionArrayBuilder::allocate(count, policy.policy())?;
        for _ in 0..count {
            policy.check_live()?;
            let from_left = match (left.head(), right.head()) {
                (Some(a), Some(b)) => !a
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .key
                    .compare(&b.borrow().as_ref().unwrap().key, policy)?
                    .is_gt(),
                (Some(_), None) => true,
                (None, Some(_)) => false,
                (None, None) => unreachable!("exact source count"),
            };
            let value = if from_left {
                left.take_head()
            } else {
                right.take_head()
            };
            merged.push(RefCell::new(Some(value)))?;
        }
        policy.check_live()?;
        Ok(Self {
            slots: merged.seal()?,
            used: count,
            cursor: Cell::new(0),
        })
    }
}
