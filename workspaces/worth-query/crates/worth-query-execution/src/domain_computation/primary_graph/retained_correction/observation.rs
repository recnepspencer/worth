//! Borrowing a current receiver yields retained DATA, never its ports or pins.
use super::catalog::MaterialBody;
use super::record::{MaterialCapture, MaterialReady};
use super::*;

#[derive(Clone)]
pub(in crate::domain_computation) struct RetainedCorrectionMaterial {
    _funding: Arc<MaterialFunding>,
    pub(in crate::domain_computation::primary_graph) capture: Arc<MaterialCapture>,
    pub(in crate::domain_computation::primary_graph) ready: Arc<MaterialReady>,
}
impl CorrectionMaterialOwner {
    pub(in crate::domain_computation::primary_graph) fn admit_original_metadata(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<(), Denial> {
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        let mut matches = state
            .catalog
            .material
            .values()
            .filter(|entry| match &entry.body {
                MaterialBody::Ready { ready, .. } => &ready.commit == commit,
                MaterialBody::Expired { commit: expired } => expired == commit,
                MaterialBody::Captured(_) | MaterialBody::Abandoned => false,
            });
        if matches.next().is_none() {
            return Err(Denial::Unavailable);
        }
        if matches.next().is_some() {
            return Err(Denial::OriginalMismatch);
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn observe_original(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<RetainedCorrectionMaterial, Denial> {
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        let mut found = None;
        for entry in state.catalog.material.values() {
            match &entry.body {
                MaterialBody::Ready { capture, ready } if &ready.commit == commit => {
                    if found.is_some() {
                        return Err(Denial::OriginalMismatch);
                    }
                    found = Some(RetainedCorrectionMaterial {
                        _funding: Arc::clone(&state.funding),
                        capture: Arc::clone(capture),
                        ready: Arc::clone(ready),
                    });
                }
                MaterialBody::Expired { commit: expired } if expired == commit => {
                    return Err(Denial::RetainedMaterialExpired)
                }
                MaterialBody::Captured(_)
                | MaterialBody::Ready { .. }
                | MaterialBody::Expired { .. }
                | MaterialBody::Abandoned => {}
            }
        }
        found.ok_or(Denial::Unavailable)
    }
}

impl RetainedCorrectionMaterial {
    pub(in crate::domain_computation::primary_graph) fn admit_recovery_admission<
        Schema: worth_query_installation::facade::ApplicationSchema,
        Operation,
        Input,
        Scope,
        InputBinding,
    >(
        &self,
        admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        binding: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
        input: &worth_query_declaration::facade::application_operation::ApplicationEncodedInput<
            InputBinding,
        >,
    ) -> Result<
        (),
        crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenialKind,
    >
    where
        InputBinding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding<
                Value = Input,
            >,
        Input: serde::Serialize,
    {
        let scope = admission.operation_scope_binding();
        let matches = |actual: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding, recorded: worth_relational::facade::identity::EntityId| actual.partition_id() == recorded.partition_value() && actual.local_slot() == recorded.local_slot_value() && actual.generation() == recorded.generation_value();
        use crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleDenialKind as Kind;
        if !matches(scope.principal(), self.capture.principal)
            || !matches(scope.scope(), self.capture.scope_entity)
        {
            return Err(Kind::ForeignPrincipal);
        }
        let canonical = input
            .capture_for_retention()
            .map_err(|_| Kind::CanonicalDerivationDenied)?;
        if input.identity() != &self.capture.input_identity
            || !binding.is_for_mutation_binding(&self.capture.mutation)
            || canonical.recorded_bytes() != self.capture.input.as_slice()
            || admission
                .governed_input_identity()
                .is_some_and(|identity| identity != input.identity())
        {
            return Err(Kind::GovernedInputMismatch);
        }
        if binding.key_identity() != &self.capture.key
            || binding.intent_identity() != &self.capture.intent_identity
        {
            return Err(Kind::ForeignIdempotencyRead);
        }
        Ok(())
    }
}
