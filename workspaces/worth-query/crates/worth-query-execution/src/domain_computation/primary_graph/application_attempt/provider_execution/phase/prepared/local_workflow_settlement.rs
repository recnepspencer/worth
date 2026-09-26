//! Join a locally performed mutation and its Query-owned settlement before
//! either effect is lowered into the Relational candidate.

use super::WorthQueryProviderAttemptPreparation;
use crate::domain_computation::primary_graph::application_attempt::{
    effect_program::{
        admit_workflow_settlement_effects, PlatformEffectDemand,
        WorthQueryCandidateValidatorWorkAdmission,
    },
    workflow_transition_program::{receipt_identity_from_outcome, transition_entity_in_receipt},
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationCommitOutcomeIdentity,
};
use crate::domain_computation::primary_graph::workflow::instance::visit_workflow_operation_settlement_facts;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleAxis;

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
        if self.workflow_settlement.is_none() {
            return Ok((self, None));
        }
        if admission
            .allowed_graph_contract()
            .external_effect()
            .is_declared()
        {
            // External custody is not performed at product publication. Its
            // existing dispatch/recovery owner settles the operation later.
            return Ok((self, None));
        }
        let binding = self
            .workflow_settlement
            .take()
            .expect("the local guarded binding was checked above");
        let receipt_identity = receipt_identity_from_outcome(
            admission.runtime_authority().as_u64(),
            outcome_identity.get(),
            admission.operation_authority_identity_bytes(),
        );
        let mut settlement_effects = Vec::new();
        visit_workflow_operation_settlement_facts(
            &binding.workflow_layout,
            &binding.settlement_basis,
            &receipt_identity,
            |effect| {
                settlement_effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        let mut demand = PlatformEffectDemand::default();
        for effect in &settlement_effects {
            demand.observe(effect)?;
        }
        let platform_work = admit_workflow_settlement_effects(admission, demand)?
            .materialize(&settlement_effects)?
            .maximum_work()
            .ok_or_else(work_denial)?;
        let handler_work = self
            .validator_work_admission
            .maximum_work()
            .ok_or_else(work_denial)?;
        let envelope = admission
            .allowed_graph_contract()
            .execution_strategy()
            .expect("installed application operation has one execution strategy")
            .envelope();
        let combined_work = combined_work_within_ceiling(
            handler_work,
            platform_work,
            envelope.scale_ceiling(WorthQuerySemanticScaleAxis::WorkItems),
        )?;
        self.validator_work_admission = WorthQueryCandidateValidatorWorkAdmission::Reserved {
            maximum_work: combined_work,
        };
        self.effects.extend(settlement_effects);
        self.effect_posture =
            crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::ApplicationWithPlatform;
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

fn work_denial() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
        "guarded workflow settlement exceeds its installed work ceiling",
    )
}

fn combined_work_within_ceiling(
    handler_work: usize,
    platform_work: usize,
    ceiling: u64,
) -> Result<usize, WorthQueryApplicationAttemptDenial> {
    let combined = handler_work
        .checked_add(platform_work)
        .ok_or_else(work_denial)?;
    if !u64::try_from(combined).is_ok_and(|work| work <= ceiling) {
        return Err(work_denial());
    }
    Ok(combined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::primary_graph::application_attempt::{
        WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
        WorthQueryApplicationCommitDenialStage,
    };

    #[test]
    fn guarded_work_combines_handler_and_settlement_and_denies_one_over() {
        assert_eq!(combined_work_within_ceiling(512, 13, 525).unwrap(), 525);
        let denial = combined_work_within_ceiling(512, 13, 524).unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
        );
        let commit_denial = WorthQueryApplicationCommitDenial::workflow_settlement_denied(&denial);
        assert_eq!(
            commit_denial.kind(),
            WorthQueryApplicationCommitDenialKind::WorkflowSettlementDenied {
                kind: WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded,
            }
        );
        assert_eq!(
            commit_denial.stage(),
            WorthQueryApplicationCommitDenialStage::ResourceAdmission
        );
        assert!(combined_work_within_ceiling(usize::MAX, 1, u64::MAX).is_err());
    }
}
