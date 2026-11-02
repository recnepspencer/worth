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
    /// Retained custody is read availability, not an instruction to publish all
    /// its records. Admission still certifies the complete available input.
    pub fn prepare_content_reads_with_retained(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        authored: &RelationalContentAttachments,
        retained: &RelationalContentAttachments,
    ) -> Result<RelationalContentReadClosure, Denial> {
        let mut available = authored.clone();
        available.attach_retained(execution, retained)?;
        self.prepare_content_reads(execution, &available)
    }

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
    /// Preserve the caller's complete authored staging, adding only resolved
    /// historical custody absent from it and the genuine selected root.
    /// Work depends on authored records and resolved identities, not unused
    /// availability. Final candidate closure remains the publication authority.
    pub fn attachments_for_reads(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        authored: &RelationalContentAttachments,
    ) -> Result<RelationalContentAttachments, Denial> {
        control::run(execution, |_control| {
            let mut identities = ExecutionOrderedMap::<ContentRefId, ()>::default();
            for record in authored.records() {
                super::directory::visit_directory(execution, identities.len())?;
                identities
                    .insert_cloned_native_for_request(&record.identity(), &(), *execution)
                    .map_err(super::directory::map_denial)?;
            }
            let mut selected = authored.clone();
            for (identity, frame) in self.resolved.iter() {
                super::directory::visit_directory(execution, identities.len())?;
                if identities.contains_key(identity)
                    || self
                        .basis
                        .inner
                        .root
                        .retained_content()
                        .find(execution, *identity)?
                        .is_some()
                {
                    continue;
                }
                selected.attach(execution, frame)?;
                super::directory::visit_directory(execution, identities.len())?;
                identities
                    .insert_cloned_native_for_request(identity, &(), *execution)
                    .map_err(super::directory::map_denial)?;
            }
            Ok(selected)
        })
    }

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
