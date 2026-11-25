//! Borrowing a current receiver yields retained DATA, never its ports or pins.
use super::catalog::MaterialBody;
use super::record::{MaterialCapture, MaterialReady};
use super::*;

#[derive(Clone)]
pub struct RetainedCorrectionMaterial {
    _funding: Arc<MaterialFunding>,
    pub capture: Arc<MaterialCapture>,
    pub ready: Arc<MaterialReady>,
}
impl CorrectionMaterialOwner {
    pub fn admit_original_metadata(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
        member: crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember,
    ) -> Result<(), Denial> {
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        let mut matches = state
            .catalog
            .material
            .iter()
            .filter(|((_, writer), _)| *writer == member)
            .filter(|(_, entry)| match &entry.body {
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
    pub fn has_captured_original(
        &self,
        original: worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
    ) -> bool {
        let state = self.state();
        state.pending.is_none()
            && state
                .catalog
                .original_entries(original)
                .any(|(_, entry)| matches!(entry.body, MaterialBody::Captured(_)))
    }
    pub fn observe_original(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
        member: crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember,
    ) -> Result<RetainedCorrectionMaterial, Denial> {
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        let mut found = None;
        let writers = state.catalog.material.iter();
        for (_, entry) in writers.filter(|((_, writer), _)| *writer == member) {
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
        found.ok_or_else(|| {
            let other_member = state.catalog.material.values().any(|entry| match &entry.body {
                MaterialBody::Ready { ready, .. } => &ready.commit == commit,
                MaterialBody::Expired { commit: expired } => expired == commit,
                MaterialBody::Captured(_) | MaterialBody::Abandoned => false,
            });
            if other_member { Denial::MemberUnavailable } else { Denial::Unavailable }
        })
    }
    pub fn observe_capture_for_replay(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
        member: crate::domain_computation::application_aftermath::WorthQueryCorrectionSourceMember,
        load_originals: impl FnOnce() -> Result<
            worth_runtime_world::facade::WorldProgramActivationHistory,
            Denial,
        >,
    ) -> Result<Arc<MaterialCapture>, Denial> {
        match self.observe_original(commit, member) {
            Ok(retained) => return Ok(retained.capture),
            Err(Denial::Unavailable | Denial::MemberUnavailable) => {}
            Err(cause) => return Err(cause),
        }
        let originals = load_originals()?;
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        let mut found = None;
        for ((identity, writer), entry) in &state.catalog.material {
            let MaterialBody::Captured(capture) = &entry.body else {
                continue;
            };
            if *writer != member {
                continue;
            }
            let Some(original) = originals.original_state(*identity) else {
                continue;
            };
            if !matches!(
                original.fate(),
                worth_foundational::facade::WorldOriginalFate::Published
                    | worth_foundational::facade::WorldOriginalFate::Continued
            ) {
                continue;
            }
            if !originals.original_observations().any(|obs| {
                obs.identity() == original.reserved_commit()
                    && obs.relational_commit() == Some(commit.commit_id)
            }) {
                continue;
            }
            if found.is_some() {
                return Err(Denial::OriginalMismatch);
            }
            found = Some(Arc::clone(capture));
        }
        found.ok_or(Denial::Unavailable)
    }
}

impl RetainedCorrectionMaterial {
    pub fn admit_recovery_admission<
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
