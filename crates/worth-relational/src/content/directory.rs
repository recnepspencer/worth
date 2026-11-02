//! Root-owned immutable reachability, with a separately funded persistent directory.
use super::{
    control, frame::layout, RelationalContentAttachments, RelationalContentDenial as Denial,
    RelationalContentFormatRegistry, RelationalContentFrame,
};
use worth_execution::{ExecutionOrderedMap, ExecutionOrderedMapDenial, ExecutionRequest};
use worth_foundational::facade::{ContentFormat, ContentRefId};

#[derive(Clone, Debug, Default)]
pub(crate) struct ContentDirectory {
    frames: ExecutionOrderedMap<ContentRefId, RelationalContentFrame>,
}

impl ContentDirectory {
    pub(crate) fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
    pub(crate) fn prepare(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        formats: &RelationalContentFormatRegistry,
        attachments: &RelationalContentAttachments,
    ) -> Result<Self, Denial> {
        if attachments.is_empty() {
            return Ok(self.clone());
        }
        control::run(execution, |control| {
            let mut frames = self.frames.clone();
            let mut total = 0_usize;
            for record in attachments.records() {
                total = total
                    .checked_add(record.bytes().len())
                    .ok_or(Denial::ArithmeticOverflow)?;
                if total > super::MAXIMUM_STAGED_CONTENT_BYTES {
                    return Err(Denial::SizeLimit {
                        observed: total,
                        maximum: super::MAXIMUM_STAGED_CONTENT_BYTES,
                    });
                }
                let (name, version, _) = layout::inspect_paid(record.bytes(), control)?;
                control.checkpoint(formats.lookup_bound())?;
                let codec = formats.format_codec(name, version)?;
                let frame = RelationalContentFrame::readmit_record(execution, codec, record)?;
                visit_directory(execution, frames.len())?;
                if let Some(prior) = frames.get(&frame.identity()) {
                    prior.same_content(execution, &frame)?;
                }
                frames
                    .insert_cloned_native_for_request(&frame.identity(), &frame, *execution)
                    .map_err(map_denial)?;
            }
            Ok(Self { frames })
        })
    }

    pub(crate) fn read(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
        expected: &ContentFormat,
    ) -> Result<RelationalContentFrame, Denial> {
        visit_directory(execution, self.frames.len())?;
        let frame = self.frames.get(&identity).ok_or(Denial::MissingContent)?;
        if frame.format().name() != expected.name() {
            return Err(Denial::FormatMismatch);
        }
        if frame.format().version() != expected.version() {
            return Err(Denial::UnsupportedVersion);
        }
        Ok(frame.clone())
    }
}

fn visit_directory(execution: &ExecutionRequest<'_, '_>, count: usize) -> Result<(), Denial> {
    let height = 2 * (usize::BITS - count.leading_zeros()) as u64 + 2;
    control::run(execution, |control| {
        control.checkpoint(height * size_of::<ContentRefId>() as u64)
    })
}

pub(super) fn map_denial(cause: ExecutionOrderedMapDenial) -> Denial {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    use worth_execution::ExecutionAllocationDenialClass as Class;
    Denial::Execution(match cause {
        ExecutionOrderedMapDenial::Request(cause) => cause.into(),
        ExecutionOrderedMapDenial::CapacityOverflow => Cause::ChargedBytesOverflow,
        ExecutionOrderedMapDenial::Allocation(cause) => match cause.classify() {
            Class::Request(cause) => cause.into(),
            Class::Physical(_) => Cause::AllocationFailed,
        },
    })
}
