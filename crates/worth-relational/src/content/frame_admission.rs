//! Revalidate actual records before any transaction or reconstructed root uses them.
use super::{
    control,
    directory::{map_denial, visit_directory},
    frame::layout,
    RelationalContentAttachments, RelationalContentDenial as Denial,
    RelationalContentFormatRegistry, RelationalContentFrame,
};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_foundational::facade::ContentRefId;

#[derive(Debug, Default)]
pub(super) struct ContentFrameAdmission(ExecutionOrderedMap<ContentRefId, RelationalContentFrame>);
impl ContentFrameAdmission {
    pub(super) fn admit(
        execution: &ExecutionRequest<'_, '_>,
        formats: &RelationalContentFormatRegistry,
        attachments: &RelationalContentAttachments,
    ) -> Result<Self, Denial> {
        control::run(execution, |control| {
            let mut frames = ExecutionOrderedMap::<ContentRefId, RelationalContentFrame>::default();
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
                Self::insert(execution, formats, record, &mut frames, control)?;
            }
            Ok(Self(frames))
        })
    }
    pub(super) fn readmit_closure(
        execution: &ExecutionRequest<'_, '_>,
        formats: &RelationalContentFormatRegistry,
        records: &RelationalContentAttachments,
    ) -> Result<Self, Denial> {
        control::run(execution, |control| {
            let mut frames = ExecutionOrderedMap::default();
            let mut previous = None;
            for record in records.records() {
                if previous.is_some_and(|prior| prior >= record.identity()) {
                    return Err(Denial::NonCanonicalContent);
                }
                previous = Some(record.identity());
                Self::insert(execution, formats, record, &mut frames, control)?;
            }
            Ok(Self(frames))
        })
    }
    fn insert(
        execution: &ExecutionRequest<'_, '_>,
        formats: &RelationalContentFormatRegistry,
        record: &super::RelationalContentRecord,
        frames: &mut ExecutionOrderedMap<ContentRefId, RelationalContentFrame>,
        control: &mut super::RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), Denial> {
        let (name, version, _) = layout::inspect_paid(record.bytes(), control)?;
        control.checkpoint(formats.lookup_bound())?;
        let format = formats.format_codec(name, version)?;
        let frame = RelationalContentFrame::readmit_record(execution, format, record)?;
        visit_directory(execution, frames.len())?;
        if let Some(prior) = frames.get(&frame.identity()) {
            control.checkpoint(1)?;
            let comparison = frame.duplicate_comparison(execution, format)?;
            prior.same_content(execution, &comparison)?;
        }
        frames
            .insert_cloned_native_for_request(&frame.identity(), &frame, *execution)
            .map_err(map_denial)?;
        Ok(())
    }
    pub(super) fn visit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        callback: &mut dyn FnMut(&RelationalContentFrame) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        for frame in self.0.values() {
            control::run(execution, |control| control.checkpoint(1))?;
            callback(frame)?;
        }
        Ok(())
    }
    pub(super) fn find(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
    ) -> Result<Option<RelationalContentFrame>, Denial> {
        visit_directory(execution, self.0.len())?;
        Ok(self.0.get(&identity).cloned())
    }
}
