//! Existing transaction and admitted-root authorities are the only live owners.
use super::{RelationalContentDenial as Denial, RelationalContentFrame};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{CanonicalDigestId, ContentRefId};
use worth_relational_contracts::internal_port::content::registry::RelationalContentFormatRegistryOwnerPort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

impl crate::mvcc::BranchBoundRelationalTransaction {
    /// Attach through this transaction's actual schema, atomically and before effects.
    pub fn attach_content(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        attachments: &super::RelationalContentAttachments,
    ) -> Result<(), Denial> {
        let admitted = super::frame_admission::ContentFrameAdmission::admit(
            execution,
            &(*self.schema_authority.registry().owner_content_formats()),
            attachments,
        )?;
        let mut next = self.content.clone();
        admitted.visit(execution, &mut |frame| next.attach(execution, frame))?;
        self.content = next;
        self.last_merged_plan = None;
        Ok(())
    }
    pub fn admit_content(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        format: CanonicalDigestId,
        input: &dyn std::any::Any,
    ) -> Result<ContentRefId, Denial> {
        super::control::run(execution, |control| {
            control.checkpoint(
                (*self.schema_authority.registry().owner_content_formats()).lookup_bound(),
            )?;
            let codec = (*self.schema_authority.registry().owner_content_formats()).get(format)?;
            let frame = RelationalContentFrame::encode(execution, codec, input)?;
            self.content.attach(execution, &frame)?;
            self.last_merged_plan = None;
            Ok(frame.identity())
        })
    }
}
