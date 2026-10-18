//! Exact semantic-call reach with directly funded persistent paths.
use super::*;
use worth_execution::{
    ChargedBytes, ExecutionAllocationPolicy, ExecutionOrderedMap, ExecutionOrderedMapDenial,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct ReachedEntity(EntityId);
impl ChargedBytes for ReachedEntity {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
#[derive(Clone, Default)]
struct Calls(ExecutionOrderedMap<ComputationRead, ()>);
// A cloned handle owns no new nested allocation. Its original node grants
// travel with the root; the enclosing retention descriptor counts those nodes.
impl ChargedBytes for Calls {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
#[derive(Clone, Default)]
pub(super) struct ReachedCalls {
    entries: ExecutionOrderedMap<ReachedEntity, Calls>,
    call_bytes: u64,
}
impl ReachedCalls {
    pub(super) fn bytes(&self) -> u64 {
        self.entries
            .retained_bytes()
            .saturating_add(self.call_bytes)
    }
    pub(super) fn contains_in(
        &self,
        entity: EntityId,
        start: ComputationRead,
        end: Option<ComputationRead>,
    ) -> bool {
        self.entries
            .get(&ReachedEntity(entity))
            .is_some_and(|value| {
                value
                    .0
                    .first_at_or_after(&start)
                    .is_some_and(|(read, _)| end.is_none_or(|end| *read < end))
            })
    }
    pub(super) fn insert(
        &mut self,
        entity: EntityId,
        read: ComputationRead,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), ExecutionOrderedMapDenial> {
        let key = ReachedEntity(entity);
        let mut value = self.entries.get(&key).cloned().unwrap_or_default();
        let old = value.0.retained_bytes();
        value.0.insert_cloned_in(&read, &(), policy)?;
        self.call_bytes = self
            .call_bytes
            .checked_sub(old)
            .and_then(|v| v.checked_add(value.0.retained_bytes()))
            .ok_or(ExecutionOrderedMapDenial::CapacityOverflow)?;
        self.entries.insert_cloned_in(&key, &value, policy)
    }
    pub(super) fn remove(
        &mut self,
        entity: EntityId,
        read: ComputationRead,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), ExecutionOrderedMapDenial> {
        let key = ReachedEntity(entity);
        let mut value = self.entries.get(&key).expect("recorded call reach").clone();
        let old = value.0.retained_bytes();
        value.0.remove_in(&read, policy)?;
        self.call_bytes = self
            .call_bytes
            .checked_sub(old)
            .and_then(|v| v.checked_add(value.0.retained_bytes()))
            .ok_or(ExecutionOrderedMapDenial::CapacityOverflow)?;
        if value.0.is_empty() {
            self.entries.remove_in(&key, policy).map(|_| ())
        } else {
            self.entries.insert_cloned_in(&key, &value, policy)
        }
    }
}
