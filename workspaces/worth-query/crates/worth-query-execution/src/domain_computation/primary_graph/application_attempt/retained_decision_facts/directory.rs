use super::{run::Run, StoreDenial};
use std::cell::RefCell;
use worth_execution::{ExecutionArray, ExecutionArrayBuilder};

pub(in crate::domain_computation::primary_graph) struct RunDirectory<T> {
    // None only during replacement in a failed/owned attempt, never a fallback.
    pub(in crate::domain_computation::primary_graph) slots:
        Option<ExecutionArray<RefCell<Option<Run<T>>>>>,
}
impl<T> RunDirectory<T> {
    pub(in crate::domain_computation::primary_graph) fn new(
        policy: super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        policy.check_live()?;
        let mut slots = ExecutionArrayBuilder::allocate(1, policy.policy())?;
        slots.push(RefCell::new(None))?;
        Ok(Self {
            slots: Some(slots.seal()?),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn ensure(
        &mut self,
        level: usize,
        policy: super::StorageControl<'_, '_>,
    ) -> Result<(), StoreDenial> {
        let current = self.slots.as_ref().unwrap().len();
        if level < current {
            return Ok(());
        }
        let needed = level.checked_add(1).ok_or(StoreDenial::Representability)?;
        let count = current
            .checked_mul(2)
            .ok_or(StoreDenial::Representability)?
            .max(needed);
        policy.check_live()?;
        let mut replacement = ExecutionArrayBuilder::allocate(count, policy.policy())?;
        let old = self.slots.take().unwrap();
        let mut old = old.into_iter();
        for slot in old.by_ref() {
            policy.check_live()?;
            replacement.push(slot)?;
        }
        for _ in current..count {
            policy.check_live()?;
            replacement.push(RefCell::new(None))?;
        }
        self.slots = Some(replacement.seal()?);
        // Exhaustion did not release old backing; seal preceded this drop.
        drop(old);
        Ok(())
    }
}
