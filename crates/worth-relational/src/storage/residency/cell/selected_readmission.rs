//! A selected directory transfers its existing member custody into funded cells.
use super::*;
use crate::durability::data::{RelationalCheckpointDirectory, RelationalRecoveryStop as Stop};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

impl RelationalRegionResidency {
    pub(crate) fn from_selected_directory(
        mut cold: RelationalColdRegion,
        directory: &RelationalCheckpointDirectory,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Stop> {
        crate::execution::check_request(request).map_err(Stop::Request)?;
        let source = cold
            .source
            .take()
            .ok_or_else(|| directory_error("cold region source missing"))?;
        if !Arc::ptr_eq(&source, &directory.source)
            || !Arc::ptr_eq(&cold.history_members, &directory.history_members)
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
                    leaf: crate::durability::data::RelationalCheckpointRegion {
                        kind_authority: cold.kind_authority,
                        partition_id: cold.partition_id,
                        content_digest: cold.content_digest,
                        entity_slots: cold.entity_slots,
                        relation_slots: cold.relation_slots,
                        unavailable_records: cold.unavailable_records,
                        head: cold.head,
                        history: cold.history,
                    },
                },
                *request,
            )
            .map_err(crate::durability::checkpoint_history::capture::map_stop)?;
        // The original directory already admitted this complete semantic member
        // roster. Sharing its intrinsic token performs no second registration,
        // dependency walk or member read. Each cell retires its actual share.
        let physical = crate::durability::medium::NativeSourceMemberState::new(
            source,
            directory.member_basis(),
        );
        Ok(Arc::new(Self {
            partition_id: cold.partition_id,
            checkpoint_bindings: Mutex::new(bindings),
            backing: Some(cold),
            head_readmission: None,
            raw_head: Mutex::new(None),
            raw_full: Mutex::new(None),
            state: Mutex::new(RegionState::new(ResidencyState::Cold, Some(physical))),
            damage: OnceLock::new(),
            leases: AtomicUsize::new(0),
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
    crate::durability::checkpoints::region_directory::corrupt(detail).into()
}
