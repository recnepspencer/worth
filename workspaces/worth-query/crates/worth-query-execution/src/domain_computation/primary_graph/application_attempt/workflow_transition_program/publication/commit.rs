use worth_foundational::facade::{AspectValue, InternedString};
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
        prepared: PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        idempotency: super::super::super::WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProgressOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        match prepared.resolve_transition_replay(self, idempotency) {
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
            assessment,
            supporting_identity,
            operation_receipt_identity,
            approval,
            approval_identity,
        ) = match prepared {
            PreparedWorkflowAdvance::Transition {
                program,
                program_revision,
                transition_identity,
                transition_identity_bytes,
                transition_identity_locator,
                node_path,
                assessment,
                supporting_identity,
                operation_receipt_identity,
                approval,
                approval_identity,
                ..
            } => (
                program,
                program_revision,
                transition_identity,
                transition_identity_bytes,
                transition_identity_locator,
                node_path,
                assessment,
                supporting_identity,
                operation_receipt_identity,
                approval,
                approval_identity,
            ),
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
        let Some(presented) = self
            .installed_program_support()
            .and_then(|support| support.present(&program_revision))
        else {
            return WorkflowProgressOutcome::Application(
                super::super::super::WorthQueryApplicationCommitOutcome::Denied(
                    super::super::super::WorthQueryApplicationCommitDenial::application_program_required(),
                ),
            );
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
            &presented,
            program,
            idempotency,
        );
        match outcome {
            super::super::super::WorthQueryApplicationCommitOutcome::Committed(receipt) => {
                self.primary_provider.graph.with_runtime(|runtime| {
                    project(
                        runtime,
                        receipt,
                        transition_identity,
                        transition_identity_locator,
                        node_path,
                        assessment,
                        approval,
                        operation_receipt_identity,
                        false,
                    )
                })
            }
            super::super::super::WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => {
                self.primary_provider.graph.with_runtime(|runtime| {
                    project(
                        runtime,
                        receipt,
                        transition_identity,
                        transition_identity_locator,
                        node_path,
                        assessment,
                        approval,
                        operation_receipt_identity,
                        true,
                    )
                })
            }
            other => WorkflowProgressOutcome::Application(other),
        }
    }
}

pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn project(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    receipt: super::super::super::WorthQueryApplicationCommitReceipt,
    transition_identity: String,
    transition_identity_locator: worth_foundational::facade::AspectFieldLocator,
    node_path: String,
    assessment: Option<PreparedWorkflowAssessmentProjection>,
    approval: Option<PreparedWorkflowApprovalProjection>,
    operation_receipt_identity: Option<[u8; 32]>,
    replayed: bool,
) -> WorkflowProgressOutcome {
    let expected = AspectValue::String(InternedString::Raw(transition_identity));
    let candidates = receipt
        .committed_changes()
        .entity_changes()
        .filter(|(_, change)| {
            *change == worth_relational::facade::publication::RecordStructuralChange::Created
        })
        .filter_map(|(entity, _)| {
            (receipt
                .committed_changes()
                .committed_field_values(entity, &[&transition_identity_locator])
                == Some(vec![expected.clone()]))
            .then_some(entity)
        })
        .collect::<Vec<_>>();
    let [transition] = candidates.as_slice() else {
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
        Some(approval) => match project_approval(runtime, &receipt, *transition, approval) {
            Some(approval) => Some(approval),
            None => return WorkflowProgressOutcome::ProjectionDenied(receipt),
        },
        None => None,
    };
    WorkflowProgressOutcome::Completed(PerformedWorkflowTransition {
        transition: *transition,
        node_path,
        receipt,
        replayed,
        assessment_evidence,
        approval,
        operation_receipt_identity,
    })
}

fn project_approval(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    receipt: &super::super::super::WorthQueryApplicationCommitReceipt,
    transition: worth_relational::facade::identity::EntityId,
    approval: PreparedWorkflowApprovalProjection,
) -> Option<PerformedWorkflowApproval> {
    let expected = AspectValue::String(InternedString::Raw(approval.identity.clone()));
    let candidates = receipt
        .committed_changes()
        .entity_changes()
        .filter(|(_, change)| {
            *change == worth_relational::facade::publication::RecordStructuralChange::Created
        })
        .filter_map(|(entity, _)| {
            (receipt
                .committed_changes()
                .committed_field_values(entity, &[&approval.identity_locator])
                == Some(vec![expected.clone()]))
            .then_some(entity)
        })
        .collect::<Vec<_>>();
    let [entity] = candidates.as_slice() else {
        return None;
    };
    let locators = approval
        .fields
        .iter()
        .map(|(locator, _)| locator)
        .collect::<Vec<_>>();
    let expected_fields = approval
        .fields
        .iter()
        .map(|(_, value)| value.clone())
        .collect::<Vec<_>>();
    if receipt
        .committed_changes()
        .committed_field_values(*entity, &locators)
        != Some(expected_fields)
    {
        return None;
    }
    let required = approval.required_fields.iter().collect::<Vec<_>>();
    if receipt
        .committed_changes()
        .committed_field_values(*entity, &required)
        .is_none()
    {
        return None;
    }
    let version = receipt.committed_changes().commit_reference().version_id;
    let exact_targets = |kind, from, maximum_work_units| {
        runtime
            .read_truth()
            .bounded_outgoing_relations_of_kind_at_version(from, kind, version, maximum_work_units)
            .ok()
            .map(|read| {
                read.into_records()
                    .into_iter()
                    .map(|record| record.target)
                    .collect::<Vec<_>>()
            })
    };
    if exact_targets(approval.transition_relation, transition, 2).as_deref() != Some(&[*entity])
        || exact_targets(approval.proposal_relation, *entity, 2).as_deref()
            != Some(&[approval.proposal])
    {
        return None;
    }
    let mut expected_evidence = approval.evidence.to_vec();
    expected_evidence.sort_unstable();
    let Some(mut actual_evidence) = exact_targets(
        approval.evidence_relation,
        *entity,
        expected_evidence.len().saturating_mul(2).saturating_add(1),
    ) else {
        return None;
    };
    actual_evidence.sort_unstable();
    if actual_evidence != expected_evidence {
        return None;
    }
    Some(PerformedWorkflowApproval {
        entity: *entity,
        identity: approval.identity,
        decision: approval.decision,
        proposal: approval.proposal,
        evidence: approval.evidence,
        approver: approval.approver,
        purpose: approval.purpose,
        expiry: approval.expiry,
    })
}
