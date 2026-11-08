//! Path admission equals one direct admission of the same newly owned bytes.
use super::*;
use crate::storage::ordered::{edit, funding::FundingSource};
use crate::{ExecutionMemoryReservation, SerialMemoryBudget};
use std::cell::Cell;
struct Funding {
    budget: SerialMemoryBudget,
    calls: Cell<u64>,
}
impl Funding {
    fn new() -> Self {
        Self {
            budget: SerialMemoryBudget::new(None),
            calls: Cell::new(0),
        }
    }
}
impl FundingSource for Funding {
    fn uses_accounting(&self) -> bool {
        true
    }
    fn reserve_memory(
        &self,
        bytes: u64,
    ) -> Result<Option<ExecutionMemoryReservation>, ExecutionOrderedMapDenial> {
        self.calls.set(self.calls.get() + 1);
        Ok(Some(self.budget.reserve(bytes).unwrap()))
    }
}
#[test]
fn ordered_path_admission_equals_direct_allocation_at_each_tree_size() {
    for count in [1, 256, 512] {
        let mut original = ExecutionOrderedMap::default();
        for key in 0..count {
            original
                .insert_cloned_native_in(&key, &key, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
        }
        let funding = Funding::new();
        let map = ExecutionOrderedMap {
            root: edit::insert_native(&original.root, &(count - 1), &900, &funding).unwrap(),
        };
        let bytes = funding.budget.charged_memory_bytes();
        let oracle = Funding::new();
        let direct = oracle.reserve_memory(bytes).unwrap();
        assert_eq!(
            funding.calls.get(),
            oracle.calls.get(),
            "a path owns one admission, like its direct allocation oracle"
        );
        assert_eq!(
            funding.budget.charged_memory_bytes(),
            oracle.budget.charged_memory_bytes()
        );
        assert_eq!(original.get(&(count - 1)), Some(&(count - 1)));
        assert_eq!(map.get(&(count - 1)), Some(&900));
        drop(map);
        drop(direct);
        assert_eq!(
            funding.budget.charged_memory_bytes(),
            oracle.budget.charged_memory_bytes()
        );
    }
}
