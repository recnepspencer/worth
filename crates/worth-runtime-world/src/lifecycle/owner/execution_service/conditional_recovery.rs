use super::*;
use crate::publication::ConditionalDefinitionAttemptCustody;
use std::sync::Arc;
use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

impl RuntimeWorldOwnerRoot<(), (), (), (), ()> {
    pub(in crate::lifecycle) fn execute_conditional_definition_recovery(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithSignal,
        bridge: &BridgeSealedRuntimeAssembly,
        custody: &Arc<ConditionalDefinitionAttemptCustody>,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> OwnerExecutionOutcome {
        self.execute_publication(
            execution,
            prepared.into_attempt(),
            cancellation,
            SignalExecutionRequest::PublishConditionalDefinition {
                publication: SignalDefinitionPublicationAdmission::Retained(Arc::clone(custody)),
                runtime_ctx: &mut (),
                apply: |transaction: &mut worth_signal::facade::SignalTransaction<
                    '_,
                    (),
                    (),
                    (),
                    (),
                    (),
                >| {
                    let completion = bridge
                        .apply_owned_conditional_installation_extension(
                            transaction,
                            custody.take_prepared(),
                        )
                        .map_err(|denial| SignalError::invalid_input(format!("{denial:?}")))?;
                    custody.retain_applied(completion);
                    Ok(())
                },
                admit_activation: |_| custody.complete_activation(bridge),
            },
        )
    }
}
