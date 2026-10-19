use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_foundational::facade::{AspectValue, InternedString};
#[path = "commit/transition_lookup.rs"]
mod transition_lookup;
pub(in crate::domain_computation::primary_graph::application_attempt) use transition_lookup::transition_entity_in_receipt;
use worth_query_installation::facade::ApplicationSchema;

use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn compare_and_commit_workflow_advance<
        Operation: 'static,
        Input,
        Scope,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        prepared: PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        idempotency: super::super::super::WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProgressOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(cause) = phase.execution_request_for(&self.product_runtime) {
            return WorkflowProgressOutcome::Application(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause)
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit"),
            );
        }
        if let Err(denial) = prepared.validate_approval_descriptor() {
            return WorkflowProgressOutcome::AuthenticationDenied(denial);
        }
        match prepared.resolve_transition_replay(execution, self, idempotency) {
            Ok(Some(outcome)) => return outcome,
            Ok(None) => {}
            Err(denial) => return WorkflowProgressOutcome::IdempotencyDenied(denial),
        }
        let (
            program,
            program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator,
            node_path,
            terminal,
            assessment,
            supporting_identity,
            operation_receipt_identity,
            progress_update,
            approval,
            approval_identity,
            approval_authentication,
        ) = match prepared {
            PreparedWorkflowAdvance::Transition {
                program,
                program_revision,
                transition_identity,
                transition_identity_bytes,
                transition_identity_locator,
                node_path,
                terminal,
                assessment,
                supporting_identity,
                operation_receipt_identity,
                progress_update,
                approval,
                approval_identity,
                approval_authentication,
                ..
            } => (
                program,
                program_revision,
                transition_identity,
                transition_identity_bytes,
                transition_identity_locator,
                node_path,
                terminal,
                assessment,
                supporting_identity,
                operation_receipt_identity,
                progress_update,
                approval,
                approval_identity,
                approval_authentication,
            ),
            PreparedWorkflowAdvance::AwaitingProposal { required, .. } => {
                return WorkflowProgressOutcome::AwaitingProposal(required)
            }
            PreparedWorkflowAdvance::AwaitingAssessment(prepared) => {
                return WorkflowProgressOutcome::AwaitingAssessment(prepared.into_required())
            }
            PreparedWorkflowAdvance::AwaitingCondition(prepared) => {
                return WorkflowProgressOutcome::AwaitingCondition(prepared.into_required())
            }
            PreparedWorkflowAdvance::AwaitingOperation(prepared) => {
                return WorkflowProgressOutcome::AwaitingOperation(prepared.into_required())
            }
            PreparedWorkflowAdvance::AwaitingEvidence { required, .. } => {
                return WorkflowProgressOutcome::AwaitingEvidence(required)
            }
            PreparedWorkflowAdvance::AwaitingApproval { required, .. } => {
                return WorkflowProgressOutcome::AwaitingApproval(required)
            }
            PreparedWorkflowAdvance::ReplayOnly { denial, .. } => {
                return WorkflowProgressOutcome::PreparationDenied(denial)
            }
        };
        let progress_update = approval_authentication
            .as_ref()
            .map(|authentication| authentication.trusted_progress_update())
            .or(progress_update);
        match (approval.is_some(), approval_authentication) {
            (true, Some(authentication)) => {
                if !authentication.matches_basis(
                    program
                        .output_currentness_facts
                        .as_ref()
                        .map(|facts| facts.facts()),
                ) {
                    return WorkflowProgressOutcome::AuthenticationDenied(
                        worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial::WrongOwner,
                    );
                }
                if let Err(denial) = authentication.readmit(&program) {
                    return WorkflowProgressOutcome::AuthenticationDenied(denial);
                }
            }
            (true, None) | (false, Some(_)) => {
                return WorkflowProgressOutcome::AuthenticationDenied(
                    worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial::MissingSigningProof,
                );
            }
            (false, None) => {}
        }
        let presented = match self
            .installed_program_support()
            .ok_or_else(super::super::super::WorthQueryApplicationCommitDenial::application_program_required)
            .and_then(|support| support.present_for_commit(&program_revision))
        {
            Ok(presented) => presented,
            Err(denial) => {
                return WorkflowProgressOutcome::Application(WorthQueryApplicationUncommitted::Denied(
                    denial,
                ));
            }
        };
        let idempotency = match &assessment {
            Some(assessment) => idempotency
                .bind_workflow_transition(&transition_identity_bytes)
                .bind_workflow_assessment(&assessment.intent_identity),
            None => idempotency.bind_workflow_transition(&transition_identity_bytes),
        };
        let idempotency = approval_identity.as_ref().map_or(idempotency, |identity| {
            idempotency.bind_workflow_approval(identity)
        });
        let idempotency = supporting_identity
            .as_ref()
            .map_or(idempotency, |identity| {
                idempotency.bind_workflow_support(identity)
            });
        let outcome = self.compare_and_commit_application_for_program_action(
            execution,
            phase,
            &presented,
            program,
            idempotency,
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        let projected = match outcome.landed() {
            Ok((receipt, replayed)) => {
                self.primary_provider
                    .graph
                    .with_runtime(execution, |runtime| {
                        project(
                            execution,
                            runtime,
                            receipt,
                            transition_identity,
                            transition_identity_locator,
                            node_path,
                            terminal,
                            assessment,
                            approval,
                            operation_receipt_identity,
                            replayed,
                        )
                    })
            }
            Err(uncommitted) => WorkflowProgressOutcome::Application(uncommitted),
        };
        if let (Some(progress_update), WorkflowProgressOutcome::Completed(performed)) =
            (progress_update, &projected)
        {
            let committed_revision = performed.receipt().commit_reference().version_id;
            let progress_key = progress_update.key();
            let _ = self
                .primary_provider
                .graph
                .with_workflow_instance_progress_mut(progress_key, |retention| {
                    progress_update.apply(
                        retention,
                        committed_revision,
                        performed.transition(),
                        performed
                            .assessment_evidence()
                            .map(|evidence| evidence.evidence()),
                    )
                });
        }
        projected
    }
}

pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn project(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    receipt: super::super::super::WorthQueryApplicationCommitReceipt,
    transition_identity: String,
    transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
    node_path: String,
    terminal: bool,
    assessment: Option<PreparedWorkflowAssessmentProjection>,
    approval: Option<PreparedWorkflowApprovalProjection>,
    operation_receipt_identity: Option<[u8; 32]>,
    replayed: bool,
) -> WorkflowProgressOutcome {
    let Some(transition) =
        transition_entity_in_receipt(&receipt, &transition_identity, &transition_identity_locator)
    else {
        return WorkflowProgressOutcome::ProjectionDenied(receipt);
    };
    let assessment_evidence = match assessment {
        Some(assessment) => {
            let expected = AspectValue::String(InternedString::Raw(assessment.identity.clone()));
            let candidates = receipt
                .committed_changes()
                .entity_changes()
                .filter(|(_, change)| {
                    *change
                        == worth_relational::facade::publication::RecordStructuralChange::Created
                })
                .filter_map(|(entity, _)| {
                    (receipt
                        .committed_changes()
                        .committed_field_values(entity, &[&assessment.identity_locator])
                        == Some(vec![expected.clone()]))
                    .then_some(entity)
                })
                .collect::<Vec<_>>();
            let [evidence] = candidates.as_slice() else {
                return WorkflowProgressOutcome::ProjectionDenied(receipt);
            };
            Some(PerformedWorkflowAssessmentEvidence {
                evidence: *evidence,
                identity: assessment.identity,
                producer: assessment.producer,
                family: assessment.family,
                query: assessment.query,
                parameter_type: assessment.parameter_type,
                result_type: assessment.result_type,
                binding: assessment.binding,
                subject: assessment.subject,
                proposal_identity: assessment.proposal_identity,
                coverage_identity: assessment.coverage_identity,
                source_identity: assessment.source_identity,
                passing: assessment.passing,
                publication_identity: assessment.publication_identity,
                output_content_identity: assessment.output_content_identity,
            })
        }
        None => None,
    };
    let approval = match approval {
        Some(approval) => {
            match project_approval(execution, runtime, &receipt, transition, approval) {
                Ok(Some(approval)) => Some(approval),
                Ok(None) => return WorkflowProgressOutcome::ProjectionDenied(receipt),
                Err(cause) => {
                    return WorkflowProgressOutcome::ProjectionReadStopped { receipt, cause }
                }
            }
        }
        None => None,
    };
    WorkflowProgressOutcome::Completed(PerformedWorkflowTransition {
        transition,
        node_path,
        terminal,
        receipt,
        replayed,
        assessment_evidence,
        approval,
        operation_receipt_identity,
    })
}

#[path = "commit/approval_projection.rs"]
mod approval_projection;
use approval_projection::project_approval;
