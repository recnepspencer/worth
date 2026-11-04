//! Complete reconstruction checks actual fields against actual retained bytes.
use super::*;

impl ContentDirectory {
    pub(crate) fn reconstruct(
        execution: &ExecutionRequest<'_, '_>,
        schema: &RelationalSchemaRegistry,
        records: &crate::content::RelationalContentAttachments,
        retained_contracts: &[worth_foundational::facade::AspectContract],
        state: &dyn crate::storage::overlay::CurrentPartitionAccess,
        version: crate::identity::data::VersionId,
    ) -> Result<Self, Denial> {
        if records.is_empty()
            && !crate::content::reconstruction::schema_has_references(
                execution,
                schema,
                retained_contracts,
            )?
        {
            return Ok(Self::default());
        }
        let frames =
            ContentFrameAdmission::readmit_closure(execution, &schema.content_formats, records)?;
        let empty = Self::default();
        let mut directory = Self::default();
        let view = InvariantStateView::new(state, version);
        let mut resolved = ExecutionOrderedMap::default();
        let mut resolution = ContentResolution::new(
            &schema.content_formats,
            &frames,
            &empty,
            None,
            &mut resolved,
        );
        crate::content::reconstruction::visit_records(execution, state, &mut |record| {
            directory.replace_record(execution, schema, &view, record, &mut resolution)
        })?;
        // Require the exact closure: omitted bytes refuse during field traversal;
        // extraneous records cannot silently obtain retention in the new root.
        if directory
            .unreferenced_attachment(execution, records)?
            .is_some()
        {
            return Err(Denial::NonCanonicalContent);
        }
        Ok(directory)
    }
    /// Existing immutable fields/bytes remain certified. Only the incoming
    /// envelope's declared records need fresh admission and exact comparison.
    pub(crate) fn certify_records(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        schema: &RelationalSchemaRegistry,
        records: &crate::content::RelationalContentAttachments,
    ) -> Result<(), Denial> {
        let frames = ContentFrameAdmission::admit(execution, &schema.content_formats, records)?;
        for record in records.records() {
            let current = self
                .find(execution, record.identity())?
                .ok_or(Denial::MissingContent)?;
            let incoming = frames
                .find(execution, record.identity())?
                .ok_or(Denial::EncodingInconsistency)?;
            current.same_content(execution, &incoming)?;
        }
        Ok(())
    }
    pub(crate) fn retained_records(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::content::RelationalContentAttachments, Denial> {
        let mut records = crate::content::RelationalContentAttachments::default();
        self.visit_frames(execution, &mut |frame| {
            records.append_retained_record(execution, &frame.record())
        })?;
        Ok(records)
    }
    pub(crate) fn visit_frames(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        callback: &mut dyn FnMut(&RelationalContentFrame) -> Result<(), Denial>,
    ) -> Result<(), Denial> {
        let mut previous = None;
        for ((identity, record), frame) in self.occurrences.iter() {
            control::run(execution, |control| control.checkpoint(1))?;
            if record.is_none() {
                return Err(Denial::EncodingInconsistency);
            }
            if previous != Some(*identity) {
                callback(frame)?;
                previous = Some(*identity);
            }
        }
        Ok(())
    }
}
