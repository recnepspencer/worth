//! Join a locally performed mutation and its Query-owned settlement before
//! either effect is lowered into the Relational candidate.

use super::WorthQueryProviderAttemptPreparation;
use crate::domain_computation::primary_graph::application_attempt::{
    effect_program::{admit_workflow_settlement_effects, PlatformEffectDemand},
    workflow_transition_program::{receipt_identity_from_outcome, transition_entity_in_receipt},
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationCommitOutcomeIdentity,
    WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::workflow::instance::visit_workflow_operation_settlement_facts;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) struct LocalWorkflowSettlementPublication {
    progress_update:
        crate::domain_computation::primary_graph::workflow::instance::PreparedWorkflowProgressUpdate,
    transition_identity: String,
    identity_locator: worth_foundational::facade::AspectFieldLocator,
}

impl LocalWorkflowSettlementPublication {
    pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) fn maintain<
        Schema,
    >(
        self,
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: &crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitReceipt,
    ) {
        let Some(transition) = transition_entity_in_receipt(
            receipt,
            &self.transition_identity,
            &self.identity_locator,
        ) else {
            return;
        };
        let key = self.progress_update.key();
        let committed_revision = receipt.commit_reference().version_id;
        let _ = application
            .primary_provider
            .graph
            .with_workflow_instance_progress_mut(key, |retention| {
                self.progress_update
                    .apply(retention, committed_revision, transition, None)
            });
    }
}

impl WorthQueryProviderAttemptPreparation {
    pub(super) fn stage_local_workflow_settlement<Schema, Operation, Input, Scope>(
        mut self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        outcome_identity: WorthQueryApplicationCommitOutcomeIdentity,
    ) -> Result<
        (Self, Option<LocalWorkflowSettlementPublication>),
        WorthQueryApplicationAttemptDenial,
    > {
        let Some(binding) = self.workflow_settlement.take() else {
            return Ok((self, None));
        };
        let external = admission
            .allowed_graph_contract()
            .external_effect()
            .is_declared();
        let receipt_identity = receipt_identity_from_outcome(
            admission.runtime_authority().as_u64(),
            outcome_identity.get(),
            admission.operation_authority_identity_bytes(),
        );
        let mut settlement_effects = Vec::new();
        if external {
            // External custody is not performed at product publication; its
            // dispatch/recovery owner settles the operation later. The
            // instance records that custody so no cancellation disposes it.
            settlement_effects.push(WorthQueryApplicationRealizedEffect::UpdateEntity {
                entity: "workflow-instance".to_owned(),
                entity_id: binding.settlement_basis.instance(),
                fields: std::collections::BTreeMap::from([(
                    binding.workflow_layout.instance.owner_custody.clone(),
                    worth_foundational::facade::AspectValue::String(
                        worth_foundational::facade::InternedString::Raw(
                            binding.settlement_basis.identity().to_owned(),
                        ),
                    ),
                )]),
            });
        } else {
            visit_workflow_operation_settlement_facts(
                &binding.workflow_layout,
                &binding.settlement_basis,
                &receipt_identity,
                |effect| {
                    settlement_effects.push(effect);
                    Ok::<(), WorthQueryApplicationAttemptDenial>(())
                },
            )?;
        }
        let mut demand = PlatformEffectDemand::default();
        for effect in &settlement_effects {
            demand.observe(effect)?;
        }
        admit_workflow_settlement_effects(admission, demand)?.materialize(&settlement_effects)?;
        self.effects.extend(settlement_effects);
        self.effect_posture =
            crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::ApplicationWithPlatform;
        if external {
            return Ok((self, None));
        }
        let progress_update = binding
            .settlement_basis
            .prepare_progress_update(receipt_identity)?;
        Ok((
            self,
            Some(LocalWorkflowSettlementPublication {
                progress_update,
                transition_identity: binding.settlement_basis.identity().to_owned(),
                identity_locator: binding.workflow_layout.transition.identity.clone(),
            }),
        ))
    }
}
