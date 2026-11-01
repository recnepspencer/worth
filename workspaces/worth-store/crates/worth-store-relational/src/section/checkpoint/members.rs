//! Request-funded opaque physical transport, preserving canonical locator order.
use super::{PreparedRelationalSection, RelationalSectionCheckpointDenial as Denial};
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionRequest};
use worth_relational::facade::durability::{
    RelationalCheckpointMember, RelationalDurableMemberLocator as Key,
};
use worth_store::physical_runtime::{AnchoredRecordMember, AnchoredRecordMemberLocator};

#[derive(Clone, Copy)]
enum TransportMember {
    Carried(AnchoredRecordMemberLocator),
    New(usize),
}
fn allocation(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    Denial::MemberSelection(crate::RelationalStoreMemberSelectionDenial::Allocation(
        cause,
    ))
}
pub(super) fn prepare(
    section: &PreparedRelationalSection,
    previous: &[AnchoredRecordMemberLocator],
    execution: &ExecutionRequest<'_, '_>,
) -> Result<
    (
        Vec<AnchoredRecordMember>,
        Option<ExecutionMemoryReservation>,
    ),
    Denial,
> {
    let mut old = ExecutionOrderedMap::default();
    for member in previous {
        old.insert_cloned_native_for_request(
            &Key {
                sha256: member.sha256(),
                length: member.length(),
            },
            member,
            *execution,
        )
        .map_err(allocation)?;
    }
    let mut plan = ExecutionOrderedMap::default();
    for (index, member) in section.checkpoint.members().iter().enumerate() {
        let key = member.locator();
        let placed = if let Some(original) = old.get(&key) {
            TransportMember::Carried(*original)
        } else {
            match member {
                RelationalCheckpointMember::New { .. } => TransportMember::New(index),
                RelationalCheckpointMember::Carried { .. } => {
                    return Err(Denial::MissingCarriedMember)
                }
            }
        };
        plan.insert_cloned_native_for_request(&key, &placed, *execution)
            .map_err(allocation)?;
    }
    for key in section.retained.member_locators() {
        execution.consult().map_err(Denial::Execution)?;
        if !plan.contains_key(&key) {
            let placed =
                TransportMember::Carried(*old.get(&key).ok_or(Denial::MissingCarriedMember)?);
            plan.insert_cloned_native_for_request(&key, &placed, *execution)
                .map_err(allocation)?;
        }
    }
    let backing = std::alloc::Layout::array::<AnchoredRecordMember>(plan.len())
        .map_err(|_| Denial::SizeLimit)?
        .size() as u64;
    let payload = plan
        .values()
        .try_fold(0u64, |total, member| match member {
            TransportMember::Carried(_) => Some(total),
            TransportMember::New(index) => match &section.checkpoint.members()[*index] {
                RelationalCheckpointMember::New { bytes, .. } => {
                    total.checked_add(bytes.len() as u64)
                }
                RelationalCheckpointMember::Carried { .. } => None,
            },
        })
        .ok_or(Denial::SizeLimit)?;
    let memory = execution
        .reserve_retained_memory(backing.checked_add(payload).ok_or(Denial::SizeLimit)?)
        .map_err(Denial::Execution)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(plan.len())
        .map_err(Denial::Allocation)?;
    for member in plan.values() {
        execution.consult().map_err(Denial::Execution)?;
        result.push(match member {
            TransportMember::Carried(locator) => {
                AnchoredRecordMember::Carried { locator: *locator }
            }
            TransportMember::New(index) => {
                let RelationalCheckpointMember::New { locator, bytes } =
                    &section.checkpoint.members()[*index]
                else {
                    return Err(Denial::MissingCarriedMember);
                };
                let mut copied = Vec::new();
                copied
                    .try_reserve_exact(bytes.len())
                    .map_err(Denial::Allocation)?;
                for chunk in bytes.chunks(16 * 1024) {
                    execution.consult().map_err(Denial::Execution)?;
                    copied.extend_from_slice(chunk);
                }
                AnchoredRecordMember::New {
                    key: locator.sha256,
                    bytes: copied,
                }
            }
        });
    }
    Ok((result, memory))
}
