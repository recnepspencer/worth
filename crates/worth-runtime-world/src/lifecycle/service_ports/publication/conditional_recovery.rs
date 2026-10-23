use super::*;
use crate::publication::{
    PreparedConditionalDefinitionRecovery, RuntimeWorldConditionalDefinitionPublicationOutcome,
};
use crate::recovery::{
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryDenial,
    RuntimeWorldSettledRelationalAdoptionDenial as Denial,
};
use worth_runtime_bridge::facade::BridgeSealedRuntimeAssembly;

impl RuntimeWorldPublicationPort<(), (), (), (), ()> {
    /// Reserve continuation of the exact conditional definition retained by an
    /// unpublished World record. Settle any pending Relational effect through
    /// the recovery port first, and drop inspection handles before this call.
    ///
    /// The original Bridge assembly validates the retained definition. The
    /// original product head must remain current. Successful preparation holds
    /// the catalog's exclusive lease while callers reserve their downstream
    /// delivery; dropping it returns the original custody without owner work.
    /// Busy inspections, pending settlement, stale selection and unavailable
    /// retained evidence remain typed refusals.
    pub fn prepare_bridge_conditional_definition_adoption(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
        bridge: &BridgeSealedRuntimeAssembly,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<PreparedConditionalDefinitionRecovery, Denial> {
        let owner = self
            .service()
            .map_err(|denial| Denial::Recovery(denial.into()))?;
        let mut update = owner
            .state
            .recovery
            .take_record_for_update(handle)
            .map_err(Denial::Recovery)?;
        let record = update.record_mut().ok_or(Denial::Recovery(
            RuntimeWorldRecoveryDenial::CallerCapabilityLive,
        ))?;
        let (expected, relational, signal, custody) = record
            .conditional_recovery_parts()
            .map_err(Denial::Recovery)?;
        custody.admit_bridge(bridge).map_err(Denial::Recovery)?;
        let mut intent = CompositePublicationIntent::with_signal(
            relational
                .as_ref()
                .map(|_| worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary()),
        );
        if let Some(relational) = relational {
            intent = intent.with_settled_relational_adoption(relational);
        }
        if let Some(signal) = signal {
            intent = intent.with_settled_signal_adoption(signal);
        }
        let mut prepared = owner
            .prepare_publication(
                expected,
                intent.with_successor_observation(),
                cancellation,
                deadline,
            )
            .map_err(Denial::Publication)?;
        let successor = prepared.unpublished_recovery_handle();
        prepared.retain_conditional_definition_custody(Arc::clone(&custody));
        let transferred = record
            .take_conditional_definition()
            .expect("validated original custody");
        assert!(Arc::ptr_eq(&transferred, &custody));
        Ok(PreparedConditionalDefinitionRecovery {
            prepared: Some(prepared),
            custody,
            update: Some(update),
            catalog: owner.state.recovery.clone(),
            successor,
            completed: false,
        })
    }

    /// Consume a prepared continuation, adopting its settled component evidence
    /// and executing only an original Signal leg that has not yet moved.
    ///
    /// Use the same Bridge assembly supplied during preparation. `Performed`
    /// carries both the combined World publication and its installed lowering.
    /// `ProductUnpublished` transfers custody to the reserved successor record;
    /// `NoEffect` leaves the original record available for recovery or cleanup.
    /// The caller releases that original record through the recovery port once
    /// its own downstream publication and cleanup obligations permit release.
    pub fn execute_bridge_conditional_definition_adoption(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        mut recovery: PreparedConditionalDefinitionRecovery,
        bridge: &BridgeSealedRuntimeAssembly,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> RuntimeWorldConditionalDefinitionPublicationOutcome {
        use RuntimeWorldConditionalDefinitionPublicationOutcome as Outcome;
        let prepared = recovery
            .prepared
            .take()
            .expect("recovery executes its reserved successor once");
        let expected = prepared.expected_head().clone();
        let owner = match self.service() {
            Ok(owner) if expected.owner_identity() == owner.owner_identity() => owner,
            _ => {
                return Outcome::NoEffect(NoEffectCompositePublication::new(
                    NoEffectCause::OwnerUnavailable,
                    Some(expected),
                ))
            }
        };
        if recovery.custody.admit_bridge(bridge).is_err() {
            return Outcome::NoEffect(NoEffectCompositePublication::new(
                NoEffectCause::OwnerDeniedBeforeEffect,
                Some(expected),
            ));
        }
        let outcome = owner.execute_conditional_definition_recovery(
            execution,
            prepared,
            bridge,
            &recovery.custody,
            cancellation,
        );
        match owner.finish_publication(outcome, cancellation) {
            RuntimeWorldPublicationOutcome::Performed(publication) => {
                let lowering = bridge.commit_owned_conditional_installation_extension(
                    recovery.custody.take_applied(),
                );
                recovery.complete();
                Outcome::Performed {
                    publication,
                    lowering,
                }
            }
            RuntimeWorldPublicationOutcome::NoEffect(denial) => Outcome::NoEffect(denial),
            RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
                recovery.complete();
                Outcome::ProductUnpublished(
                    crate::publication::RuntimeWorldUnpublishedConditionalDefinition::new(effects),
                )
            }
        }
    }
}
