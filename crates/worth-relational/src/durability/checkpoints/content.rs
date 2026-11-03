//! Content is a Native checkpoint member through the existing physical port.
use crate::content::{
    ContentDirectory, RelationalContentAttachments, RelationalContentDenial as Denial,
    RelationalContentRecord, MAXIMUM_CONTENT_FRAME_BYTES,
};
use crate::durability::data::{
    RelationalCheckpointContent, RelationalCheckpointMember,
    RelationalNativeCheckpointCaptureDenial, RelationalRecoveryStop,
};
use crate::durability::log::native_checkpoint_codec::member_frame;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};

pub(crate) fn capture(
    execution: &ExecutionRequest<'_, '_>,
    directory: &ContentDirectory,
    members: &mut BTreeMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    current: Option<&BTreeSet<RelationalDurableMemberLocator>>,
) -> Result<Vec<RelationalCheckpointContent>, RelationalNativeCheckpointCaptureDenial> {
    let mut content = Vec::new();
    directory.visit_frames(execution, &mut |frame| {
        let locator = member_frame::locator_for_request(execution, frame.bytes())?;
        if let std::collections::btree_map::Entry::Vacant(entry) = members.entry(locator) {
            let member = if current.is_some_and(|members| members.contains(&locator)) {
                RelationalCheckpointMember::Carried { locator }
            } else {
                let mut bytes =
                    ExecutionRequestArrayBuilder::<u8>::allocate(*execution, frame.bytes().len())
                        .map_err(allocation)?;
                for chunk in frame.bytes().chunks(4096) {
                    crate::content::charge_bytes(execution, chunk.len())?;
                    bytes.extend_from_slice(chunk).map_err(allocation)?;
                }
                RelationalCheckpointMember::New {
                    locator,
                    bytes: bytes.seal().map_err(allocation)?.into_immutable_bytes(),
                }
            };
            entry.insert(member);
        }
        content.try_reserve(1).map_err(|_| {
            Denial::Execution(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
        })?;
        content.push(RelationalCheckpointContent {
            identity: frame.identity(),
            member: locator,
        });
        Ok(())
    })?;
    Ok(content)
}

pub(crate) fn read_records(
    execution: &ExecutionRequest<'_, '_>,
    content: &[RelationalCheckpointContent],
    source: &dyn RelationalDurableColdSource,
) -> Result<RelationalContentAttachments, RelationalRecoveryStop> {
    let mut records = RelationalContentAttachments::default();
    let mut previous = None;
    for entry in content {
        execution
            .check_live()
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        if previous.is_some_and(|identity| identity >= entry.identity) {
            return Err(Denial::NonCanonicalContent.into());
        }
        previous = Some(entry.identity);
        if entry.member.length > MAXIMUM_CONTENT_FRAME_BYTES as u64 {
            return Err(Denial::SizeLimit {
                observed: usize::try_from(entry.member.length)
                    .map_err(|_| Denial::ArithmeticOverflow)?,
                maximum: MAXIMUM_CONTENT_FRAME_BYTES,
            }
            .into());
        }
        // The physical provider enforces the limit before allocating. This grant
        // funds that temporary read buffer until Native's own backing is admitted.
        let _read = execution
            .reserve_retained_memory(entry.member.length)
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        let bytes = source
            .read_member(entry.member, entry.member.length)
            .map_err(|cause| match cause {
                crate::storage::data::RelationalStorageUnavailableCause::IntegrityMismatch => {
                    Denial::IntegrityMismatch
                }
                crate::storage::data::RelationalStorageUnavailableCause::MissingMember => {
                    Denial::MissingContent
                }
                cause => Denial::BackingUnavailable {
                    identity: entry.identity,
                    cause,
                },
            })?;
        if member_frame::locator_for_request(execution, &bytes)
            .map_err(RelationalRecoveryStop::Request)?
            != entry.member
        {
            return Err(Denial::IntegrityMismatch.into());
        }
        let record = RelationalContentRecord::readmit_transport(execution, entry.identity, &bytes)?;
        records.append_retained_record(execution, &record)?;
    }
    Ok(records)
}
fn allocation(cause: worth_execution::ExecutionRequestArrayDenial) -> Denial {
    let cause = match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
            return Denial::Execution(cause.into())
        }
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => cause,
    };
    use worth_execution::ExecutionAllocationDenialClass as Class;
    Denial::Execution(match cause.classify() {
        Class::Request(cause) => cause.into(),
        Class::Physical(_) => crate::execution::RelationalExecutionDenialCause::AllocationFailed,
    })
}
