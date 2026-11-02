//! Resolve each identity once per preparation, including exact collision checks.
use super::{
    control,
    directory::{map_denial, visit_directory},
    frame_admission::ContentFrameAdmission,
    ContentDirectory, RelationalContentDenial as Denial, RelationalContentFrame,
};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_foundational::facade::ContentRefId;

pub(super) struct ContentResolution<'a> {
    formats: &'a super::RelationalContentFormatRegistry,
    staged: &'a ContentFrameAdmission,
    current: &'a ContentDirectory,
    source: Option<&'a ContentDirectory>,
    resolved: &'a mut ExecutionOrderedMap<ContentRefId, RelationalContentFrame>,
}
impl<'a> ContentResolution<'a> {
    pub(super) fn new(
        formats: &'a super::RelationalContentFormatRegistry,
        staged: &'a ContentFrameAdmission,
        current: &'a ContentDirectory,
        source: Option<&'a ContentDirectory>,
        resolved: &'a mut ExecutionOrderedMap<ContentRefId, RelationalContentFrame>,
    ) -> Self {
        Self {
            formats,
            staged,
            current,
            source,
            resolved,
        }
    }
    pub(super) fn resolve(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
    ) -> Result<RelationalContentFrame, Denial> {
        visit_directory(execution, self.resolved.len())?;
        if let Some(frame) = self.resolved.get(&identity) {
            return Ok(frame.clone());
        }
        let mut chosen: Option<RelationalContentFrame> = None;
        for frame in [
            self.staged.find(execution, identity)?,
            self.current.find(execution, identity)?,
            match self.source {
                Some(source) => source.find(execution, identity)?,
                None => None,
            },
        ]
        .into_iter()
        .flatten()
        {
            if let Some(prior) = &chosen {
                prior.same_content(execution, &frame)?;
            } else {
                chosen = Some(frame);
            }
        }
        let frame = chosen.ok_or(Denial::MissingContent)?;
        control::run(execution, |control| {
            control.checkpoint(self.formats.lookup_bound())
        })?;
        let installed = self
            .formats
            .format_codec(frame.format().name(), frame.format().version())?;
        let frame = frame.certify_format(execution, installed)?;
        control::run(execution, |control| control.checkpoint(1))?;
        self.resolved
            .insert_cloned_native_for_request(&identity, &frame, *execution)
            .map_err(map_denial)?;
        Ok(frame)
    }
}
