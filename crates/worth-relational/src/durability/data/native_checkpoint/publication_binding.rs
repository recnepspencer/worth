//! Bind the exact captured data subject to its actual medium.
use super::*;
impl RelationalNativeCheckpoint {
    pub(crate) fn bind_publication(
        self,
        binding: Option<Arc<crate::durability::medium::RelationalDurableMediumBinding>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::super::RelationalNativeCheckpointCaptureDenial> {
        self.bind_original_publication(binding, None, execution)
    }
    pub(crate) fn bind_original_publication(
        mut self,
        binding: Option<Arc<crate::durability::medium::RelationalDurableMediumBinding>>,
        initial_inputs: Option<
            crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact,
        >,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::super::RelationalNativeCheckpointCaptureDenial> {
        if let Some(binding) = binding
            && self
                .manifest_publication
                .as_ref()
                .is_some_and(|publication| publication.source_matches(&binding.cold_source()))
        {
            let subject = crate::durability::medium::OpaqueCapturedCheckpointSubject::new(
                self.bytes.clone(),
                self.region.clone(),
                self.members.entries(),
                initial_inputs,
                execution,
            )?;
            self.publication = Some(Arc::new(
                crate::durability::medium::CheckpointPublication::new(
                    subject,
                    Arc::downgrade(&binding),
                    self.clone(),
                    execution,
                )?,
            ));
        }
        Ok(self)
    }
}
