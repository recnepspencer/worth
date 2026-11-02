//! One paid staged index and genuine basis serve all field checks in a batch.
use super::{
    control, directory::require_format, frame_admission::ContentFrameAdmission,
    resolution::ContentResolution, RelationalContentAttachments, RelationalContentDenial as Denial,
    RelationalContentFrame,
};
use crate::branch::AdmittedRelationalBranchBasis;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};

#[derive(Debug)]
pub struct RelationalContentReadClosure {
    basis: AdmittedRelationalBranchBasis,
    staged: ContentFrameAdmission,
    resolved: ExecutionOrderedMap<ContentRefId, RelationalContentFrame>,
}
impl AdmittedRelationalBranchBasis {
    pub fn prepare_content_reads(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        attachments: &RelationalContentAttachments,
    ) -> Result<RelationalContentReadClosure, Denial> {
        let schema = self.inner.root.retained_schema_authority();
        let staged = ContentFrameAdmission::admit(
            execution,
            &schema.registry().content_formats,
            attachments,
        )?;
        Ok(RelationalContentReadClosure {
            basis: self.clone(),
            staged,
            resolved: ExecutionOrderedMap::default(),
        })
    }
    /// Stageable encoding uses the selected owner's frozen codec, not a caller codec.
    pub fn encode_content(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        format: CanonicalDigestId,
        input: &dyn std::any::Any,
    ) -> Result<RelationalContentFrame, Denial> {
        let schema = self.inner.root.retained_schema_authority();
        control::run(execution, |control| {
            control.checkpoint(schema.registry().content_formats.lookup_bound())?;
            RelationalContentFrame::encode(
                execution,
                schema.registry().content_formats.get(format)?,
                input,
            )
        })
    }
}
impl RelationalContentReadClosure {
    pub fn read(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
        format: CanonicalDigestId,
    ) -> Result<RelationalContentFrame, Denial> {
        let root = &self.basis.inner.root;
        let schema = root.retained_schema_authority();
        let formats = &schema.registry().content_formats;
        control::run(execution, |control| {
            control.checkpoint(formats.lookup_bound())
        })?;
        let expected = formats.get(format)?;
        let mut resolution = ContentResolution::new(
            formats,
            &self.staged,
            root.retained_content(),
            None,
            &mut self.resolved,
        );
        let frame = resolution.resolve(execution, identity)?;
        require_format(execution, &frame, expected)?;
        Ok(frame)
    }
}
