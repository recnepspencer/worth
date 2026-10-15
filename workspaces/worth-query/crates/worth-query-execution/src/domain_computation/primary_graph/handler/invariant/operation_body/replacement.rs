//! Prepare selection preimages inside the body's native decision observation.
use super::{
    ApplicationMutationBinding, ApplicationSchema, DecisionReader, HandlerExecutionDenial,
};
use crate::domain_computation::primary_graph::{
    operation_body::PreparedBodyReplacements, WorthQueryAdvancementDenial,
    WorthQueryInvariantMutationTarget, WorthQueryManagedComputationResourceDenial,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreationLinkPosture as Posture, ApplicationOperationBodyProgram,
};
impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    DecisionReader<'_, '_, '_, Schema, Binding>
{
    pub(in crate::domain_computation::primary_graph) fn body_replacements(
        &mut self,
        body: &ApplicationOperationBodyProgram,
    ) -> Result<PreparedBodyReplacements<Schema>, HandlerExecutionDenial> {
        let links = body.creation().map_or(&[][..], |creation| creation.links());
        if !links
            .iter()
            .any(|link| link.posture == Posture::ReplaceSingle)
        {
            return Ok(PreparedBodyReplacements {
                targets: Vec::new(),
                _memory: None,
            });
        }
        let bytes = links
            .iter()
            .try_fold(0usize, |total, link| {
                total
                    .checked_add(size_of::<
                        Option<WorthQueryInvariantMutationTarget<Schema, ()>>,
                    >())
                    .and_then(|bytes| bytes.checked_add(link.to.len()))
            })
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| {
                HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::CapacityOverflow,
                ))
            })?;
        let execution = self.execution;
        let memory = execution.reserve(bytes).map_err(|cause| {
            HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause))
        })?;
        self.context_use.set(self.context_use.get().scope());
        let targets = execution
            .decision_stage(|context| {
                let mut targets = Vec::with_capacity(links.len());
                for link in links {
                    targets.push(if link.posture == Posture::ReplaceSingle {
                        self.reader
                            .body_creation_predecessor(self.scope, link, execution, context)?
                    } else {
                        None
                    });
                }
                Ok(targets)
            })
            .map_err(HandlerExecutionDenial::new)??;
        Ok(PreparedBodyReplacements {
            targets,
            _memory: Some(memory),
        })
    }
}
