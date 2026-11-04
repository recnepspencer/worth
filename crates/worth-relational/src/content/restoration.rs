//! Native inverse records retain only the historical bytes their actual fields reference.
use super::{
    control, ContentPublication, RelationalContentAttachments, RelationalContentDenial as Denial,
};
use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};

impl ContentPublication {
    pub(crate) fn retain_restoration_content(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        state: Option<&worth_foundational::facade::AuthoritativeRecordAspectState>,
        attachments: &mut RelationalContentAttachments,
    ) -> Result<(), Denial> {
        let Some(state) = state else {
            return Ok(());
        };
        control::run(execution, |control| {
            for (_, aspect) in state.aspects().entries() {
                control.checkpoint(1)?;
                let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
                    continue;
                };
                for (_, value) in fields.fields() {
                    control.checkpoint(1)?;
                    if let AspectValue::ContentRef(identity) = value {
                        let (directory, _) = self.historical_parts()?;
                        let frame = directory
                            .find(execution, *identity)?
                            .ok_or(Denial::MissingContent)?;
                        attachments.attach(execution, &frame)?;
                    }
                }
            }
            Ok(())
        })
    }
}
