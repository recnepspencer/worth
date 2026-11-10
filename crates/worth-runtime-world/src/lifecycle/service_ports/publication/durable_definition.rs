//! Conditional definitions consume their original durable WithSignal preparation.
use super::*;
use crate::publication::{
    AuthoredConditionalDefinitionAttemptCustody,
    RuntimeWorldConditionalDefinitionPublicationOutcome as Outcome,
    RuntimeWorldUnpublishedConditionalDefinition, WorldDurablePublicationOutcome,
    WorldPreparedPublicationWithSignal,
};
use worth_runtime_bridge::facade::BridgePreparedAuthoredDefinitionPublication;

impl RuntimeWorldPublicationPort<(), (), (), (), ()> {
    pub fn publish_durable_bridge_conditional_definition(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: WorldPreparedPublicationWithSignal,
        definition: BridgePreparedAuthoredDefinitionPublication,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Outcome {
        let completion = definition.completion();
        let mut outcome = self.execute_durable_authored_conditional_definition(
            execution,
            prepared,
            definition,
            cancellation,
        );
        loop {
            match outcome {
                WorldDurablePublicationOutcome::Performed(publication) => {
                    let output = publication
                        .component_results()
                        .signal()
                        .durable_advance_output()
                        .expect("authored conditional execution retains its actual Signal output");
                    let lowering = completion
                        .installed_lowering(output)
                        .expect("World movement follows the same original's validated projection");
                    return Outcome::Performed {
                        publication,
                        lowering,
                    };
                }
                WorldDurablePublicationOutcome::Pending(pending) => {
                    if execution.consult().is_err() || cancellation.is_cancelled() {
                        return Outcome::ProductUnpublished(
                            RuntimeWorldUnpublishedConditionalDefinition::from_effects(
                                pending.retain(),
                            ),
                        );
                    }
                    outcome = pending.advance(&execution);
                }
                WorldDurablePublicationOutcome::ProductUnpublished(custody)
                | WorldDurablePublicationOutcome::RecoveryRequired { custody, .. } => {
                    return Outcome::ProductUnpublished(
                        RuntimeWorldUnpublishedConditionalDefinition::from_effects(custody),
                    )
                }
            }
        }
    }
    pub fn execute_durable_authored_conditional_definition(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        mut prepared: WorldPreparedPublicationWithSignal,
        definition: BridgePreparedAuthoredDefinitionPublication,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> WorldDurablePublicationOutcome {
        if !definition.admits_signal_basis(prepared.expected_head().basis().signal_basis()) {
            return WorldDurablePublicationOutcome::RecoveryRequired {
                cause:crate::publication::WorldDurablePublicationDenial::BridgeDefinitionPreparation(
                    Box::new(worth_runtime_bridge::facade::BridgeDurableDefinitionPublicationDenial::SourceBasisMismatch)),
                custody:prepared.cancel(),
            };
        }
        let original = AuthoredConditionalDefinitionAttemptCustody::new(definition);
        let custody = prepared.prepared.reserve_conditional_definition_custody();
        custody.retain_authored(Arc::clone(&original));
        let owner = match self.service() {
            Ok(owner) if owner.owner_identity() == prepared.expected_head().owner_identity() => {
                owner
            }
            Ok(_) | Err(_) => {
                return WorldDurablePublicationOutcome::ProductUnpublished(prepared.cancel())
            }
        };
        let outcome = owner.execute_authored_conditional_definition_with_signal(
            execution,
            prepared.prepared,
            original,
            &mut (),
            cancellation,
            |_| Ok(()),
        );
        owner.finish_durable_publication(&execution, outcome, cancellation)
    }
}
