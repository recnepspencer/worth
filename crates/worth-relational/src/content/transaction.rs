//! Existing transaction and admitted-root authorities are the only live owners.
use super::{RelationalContentDenial as Denial, RelationalContentFrame};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};

impl crate::mvcc::BranchBoundRelationalTransaction {
    pub fn admit_content(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        format: CanonicalDigestId,
        input: &dyn std::any::Any,
    ) -> Result<ContentRefId, Denial> {
        super::control::run(execution, |control| {
            control.checkpoint(
                self.schema_authority
                    .registry()
                    .content_formats
                    .lookup_bound(),
            )?;
            let codec = self
                .schema_authority
                .registry()
                .content_formats
                .get(format)?;
            let frame = RelationalContentFrame::encode(execution, codec, input)?;
            self.content.attach(execution, &frame)?;
            self.last_merged_plan = None;
            Ok(frame.identity())
        })
    }
}

impl crate::branch::AdmittedRelationalBranchBasis {
    pub fn read_content(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: ContentRefId,
        format: CanonicalDigestId,
    ) -> Result<RelationalContentFrame, Denial> {
        super::control::run(execution, |control| {
            let schema = self.inner.root.retained_schema_authority();
            control.checkpoint(schema.registry().content_formats.lookup_bound())?;
            let codec = schema.registry().content_formats.get(format)?;
            self.inner
                .root
                .retained_content()
                .read(execution, identity, codec.descriptor())
        })
    }
}
