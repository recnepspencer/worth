//! Install actual operation/recovery and original input tickets before any step.
use super::{owner::HomeCheckpointAttempt, ApplicationHomeCheckpointDenial as Denial};
use std::sync::Arc;
use worth_store::physical_runtime::*;

pub(super) fn start(
    serving: &ServingPhysicalRuntime,
    attempt: &Arc<HomeCheckpointAttempt>,
    prepared: super::preparation::PreparedHomeInput,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<(), Denial> {
    attempt
        .native()
        .expect("unsubmitted attempt owns its capture")
        .validate_publication()
        .map_err(Denial::Native)?;
    let super::preparation::PreparedHomeInput {
        payload,
        members,
        member_memory,
        generation,
    } = prepared;
    let crate::frame_encoding::EncodedHomeFrame { bytes, memory } = payload;
    *attempt
        .input_memory
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(Arc::new(
        super::input_memory::CheckpointInputMemory::new(memory, member_memory),
    ));
    let view = context
        .retain_interruption_view(lease)
        .map_err(Denial::InterruptionView)?;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    // Signal's physical policy remains independent of the original absolute
    // execution deadline retained above; there is no clock conversion.
    let deadline = PhysicalMutationDeadline::after_milliseconds(120_000)
        .expect("fixed positive physical policy");
    let publication = AnchoredRecordPublication::new(
        AnchoredRecordName::new(crate::registry::CHECKPOINT_ANCHOR)
            .expect("static checkpoint anchor"),
        generation,
        bytes,
        crate::open::physical_admission::placement(),
        deadline,
    )
    .with_members(members);
    match records.start_publication(publication, view) {
        AnchoredRecordPublicationStart::Ready {
            operation,
            recovery,
        } => {
            attempt
                .physical
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .install(operation, recovery);
            Ok(())
        }
        AnchoredRecordPublicationStart::AlreadyPublished(receipt) => {
            attempt.selected(receipt.membership().clone());
            Ok(())
        }
        AnchoredRecordPublicationStart::Refused(cause) => Err(Denial::PhysicalPublication(cause)),
    }
}
