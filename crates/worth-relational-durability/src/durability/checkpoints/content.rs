//! Content is a Native checkpoint member through the existing physical port.
use crate::durability::data::RelationalCheckpointContent;
use crate::durability::log::native_checkpoint_codec::member_frame;
use contracts::content::denial::RelationalContentDenial as Denial;
use contracts::content::limits::MAXIMUM_CONTENT_FRAME_BYTES;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::content::attachments::{
    RelationalContentAttachments, RelationalContentAttachmentsOwnerPort as _,
};
use worth_relational_storage::internal_port::content::directory::ContentDirectory;
use worth_relational_storage::internal_port::content::frame::record::RelationalContentRecord;
use worth_relational_storage::internal_port::durability::medium::cold_source::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational_storage::internal_port::storage::data::checkpoint_member::RelationalCheckpointMember;
pub mod root_capture;

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
                    worth_relational_storage::internal_port::content::charge_bytes(execution, chunk.len())?;
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
            Denial::Execution(contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed)
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
                contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause::IntegrityMismatch => {
                    Denial::IntegrityMismatch
                }
                contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause::MissingMember => {
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
        Class::Physical(_) => {
            contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed
        }
    })
}
