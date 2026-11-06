//! Original component content owns its roster and member backing before growth.
use super::*;
use worth_execution::{ExecutionMemoryReservation, ExecutionOrderedMap};

pub(crate) struct CapturedRootContent {
    pub(crate) rows: Vec<RelationalCheckpointContent>,
    pub(crate) memory: Option<ExecutionMemoryReservation>,
}

pub(crate) fn capture(
    request: &ExecutionRequest<'_, '_>,
    directory: &ContentDirectory,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
) -> Result<CapturedRootContent, RelationalNativeCheckpointCaptureDenial> {
    let mut count = 0usize;
    directory.visit_frames(request, &mut |_| {
        count = count.checked_add(1).ok_or(Denial::ArithmeticOverflow)?;
        Ok(())
    })?;
    let bytes = std::alloc::Layout::array::<RelationalCheckpointContent>(count)
        .map_err(|_| Denial::ArithmeticOverflow)?
        .size();
    let memory = request
        .reserve_retained_memory(bytes as u64)
        .map_err(|cause| RelationalNativeCheckpointCaptureDenial::Request(cause.into()))?;
    let mut rows = Vec::with_capacity(count);
    directory.visit_frames(request, &mut |frame| {
        if rows.len() == count { return Err(Denial::EncodingInconsistency); }
        let locator = member_frame::locator_for_request(request, frame.bytes())?;
        if !members.contains_key(&locator) {
            let mut bytes = ExecutionRequestArrayBuilder::<u8>::allocate(*request, frame.bytes().len())
                .map_err(allocation)?;
            for chunk in frame.bytes().chunks(4096) {
                crate::content::charge_bytes(request, chunk.len())?;
                bytes.extend_from_slice(chunk).map_err(allocation)?;
            }
            let member = RelationalCheckpointMember::New {
                locator,
                bytes: bytes.seal().map_err(allocation)?.into_immutable_bytes(),
            };
            members.insert_cloned_native_for_request(&locator, &member, *request)
                .map_err(|cause| match cause {
                    worth_execution::ExecutionOrderedMapDenial::Request(cause) => Denial::Execution(cause.into()),
                    worth_execution::ExecutionOrderedMapDenial::Allocation(cause) => {
                        match cause.classify() {
                            worth_execution::ExecutionAllocationDenialClass::Request(cause) => Denial::Execution(cause.into()),
                            worth_execution::ExecutionAllocationDenialClass::Physical(_) => Denial::Execution(crate::execution::RelationalExecutionDenialCause::AllocationFailed),
                        }
                    }
                    worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => Denial::ArithmeticOverflow,
                })?;
        }
        rows.push(RelationalCheckpointContent { identity: frame.identity(), member: locator });
        Ok(())
    })?;
    if rows.len() != count {
        return Err(Denial::EncodingInconsistency.into());
    }
    Ok(CapturedRootContent { rows, memory })
}
