//! A selected directory transfers its existing member custody into funded cells.
use super::*;
use crate::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
use crate::durability::medium::NativeSourceMemberState;
use crate::storage::data::history_member_directory::RelationalHistoryMemberDirectory;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational_contracts::internal_port::durability::data::recovery_errors::DurabilityErrorOwnerPort as _;

impl RelationalRegionResidency {
    pub fn from_selected_directory(
        mut cold: RelationalColdRegion,
        physical: NativeSourceMemberState,
        selected_history: &Arc<RelationalHistoryMemberDirectory>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Stop> {
        crate::execution::check_request(request).map_err(Stop::Request)?;
        let source = cold
            .source
            .take()
            .ok_or_else(|| directory_error("cold region source missing"))?;
        if !physical
            .source()
            .is_some_and(|selected| Arc::ptr_eq(&source, selected))
            || !Arc::ptr_eq(&cold.history_members, selected_history)
        {
            return Err(directory_error(
                "cold region differs from its selected directory",
            ));
        }
        let memory = control_memory(request)?;
        let mut bindings = worth_execution::ExecutionOrderedMap::default();
        bindings
            .insert_cloned_native_for_request(
                &cold.schema_carrier_digest,
                &super::super::RecordedRegionCheckpoint {
                    history_members: cold.history_members.clone(),
                    leaf: cold.leaf(),
                },
                *request,
            )
            .map_err(crate::durability::data::recovery_stop::map_stop)?;
        // The original directory already admitted this complete semantic member
        // roster. Sharing its intrinsic token performs no second registration,
        // dependency walk or member read. Each cell retires its actual share.
        Ok(Arc::new(Self {
            partition_id: cold.partition_id,
            checkpoint_bindings: Mutex::new(bindings),
            backing: Some(cold),
            head_readmission: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(RegionState::new(ResidencyState::Cold, Some(physical))),
            damage: OnceLock::new(),
            symbol_aliases: OnceLock::new(),
            symbol_inheritance: OnceLock::new(),
            leases: AtomicUsize::new(0),
            working_set: OnceLock::new(),
            _control_memory: memory,
        }))
    }
}

fn control_memory(
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<Arc<ExecutionMemoryReservation>>, Stop> {
    let arc = |payload| {
        std::alloc::Layout::new::<[usize; 2]>()
            .extend(payload)
            .map(|value| value.0.pad_to_align().size())
            .map_err(|_| directory_error("region control layout overflow"))
    };
    let bytes = arc(std::alloc::Layout::new::<RelationalRegionResidency>())?
        .checked_add(arc(std::alloc::Layout::new::<ExecutionMemoryReservation>())?)
        .ok_or_else(|| directory_error("region control layout overflow"))?;
    request
        .reserve_retained_memory(bytes as u64)
        .map(|memory| memory.map(Arc::new))
        .map_err(|cause| Stop::Request(cause.into()))
}
fn directory_error(detail: &str) -> Stop {
    crate::durability::data::recovery_errors::DurabilityError::corrupt_checkpoint(detail).into()
}
