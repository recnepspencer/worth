//! Partition navigation borrows the original result-memory loan once.
use crate::execution::{denial::lease_denial, RelationalExecutionDenialCause as Cause};
use worth_execution::{ExecutionResultMemory, LeaseDenial};

pub(crate) fn partition_ids_in_result_memory(
    state: &(impl crate::storage::overlay::PartitionAccess + ?Sized),
    memory: Option<&mut ExecutionResultMemory<'_>>,
) -> Result<(Vec<crate::identity::data::PartitionId>, u64), Cause> {
    use crate::storage::overlay::{PartitionIdMemory, PartitionIdsDenial};
    let Some(memory) = memory else {
        return Ok((state.partition_ids(), 0));
    };
    struct OriginalLoan<'a, 'scope>(&'a mut ExecutionResultMemory<'scope>);
    impl PartitionIdMemory for OriginalLoan<'_, '_> {
        fn reserve(&mut self, bytes: u64) -> Result<(), worth_execution::MemoryLimitDenial> {
            let charged = self.0.bytes();
            let total = charged.checked_add(bytes).ok_or(
                worth_execution::MemoryLimitDenial::CounterOverflow {
                    requested: bytes,
                    charged,
                    level: worth_execution::MemoryLimitLevel::Declared,
                },
            )?;
            self.0.resize(total)
        }
        fn release(&mut self, bytes: u64) {
            let total = self
                .0
                .bytes()
                .checked_sub(bytes)
                .expect("the original ID vector owns its reserved backing");
            self.0
                .resize(total)
                .expect("releasing already held memory cannot refuse");
        }
    }
    let ids = state
        .partition_ids_with_memory(&mut OriginalLoan(memory))
        .map_err(|denial| match denial {
            PartitionIdsDenial::Memory(cause) => lease_denial(LeaseDenial::MemoryExhausted(cause)),
            PartitionIdsDenial::SizeOverflow => Cause::ChargedBytesOverflow,
            PartitionIdsDenial::AllocationFailed => Cause::AllocationFailed,
        })?;
    let held = u64::try_from(ids.capacity())
        .ok()
        .and_then(|capacity| {
            capacity.checked_mul(std::mem::size_of::<crate::identity::data::PartitionId>() as u64)
        })
        .ok_or(Cause::ChargedBytesOverflow)?;
    Ok((ids, held))
}
