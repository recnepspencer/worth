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
        prepared: PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        idempotency: super::super::super::WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProgressOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(denial) = prepared.validate_approval_descriptor() {
            return WorkflowProgressOutcome::AuthenticationDenied(denial);
        }
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
                if !authentication.matches_basis(program.output_currentness_facts.as_ref()) {
                    return WorkflowProgressOutcome::AuthenticationDenied(
                        worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEventDenial::WrongOwner,
                    );
                }
                if let Err(denial) = authentication.readmit() {
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
                return WorkflowProgressOutcome::Application(super::super::super::WorthQueryApplicationCommitOutcome::Denied(
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
            &presented,
            program,
            idempotency,
        );
        let projected = match outcome {
            super::super::super::WorthQueryApplicationCommitOutcome::Committed(receipt) => {
                self.primary_provider.graph.with_runtime(|runtime| {
                    project(
                        runtime,
                        receipt,
                        transition_identity,
                        transition_identity_locator,
                        node_path,
                        terminal,
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
                        terminal,
                        assessment,
                        approval,
                        operation_receipt_identity,
                        true,
                    )
                })
            }
            other => WorkflowProgressOutcome::Application(other),
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
        Some(approval) => match project_approval(runtime, &receipt, transition, approval) {
            Some(approval) => Some(approval),
            None => return WorkflowProgressOutcome::ProjectionDenied(receipt),
        },
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
    receipt
        .committed_changes()
        .committed_field_values(*entity, &required)?;
    // The committing branch's own state: a fork's approval never lands in
    // main's edition.
    let committed = runtime
        .read_truth()
        .try_project_historical_version(receipt.committed_changes().commit_reference().version_id)
        .ok()?;
    // One unit for the adjacency list and two per expected relation: an
    // unexpected extra relation exhausts the bound and fails closed.
    let exact_targets = |kind, from, expected: usize| {
        committed
            .bounded_outgoing_relations_for_frontier(
                &std::collections::BTreeSet::from([from]),
                kind,
                expected.saturating_mul(2).saturating_add(1),
            )
            .ok()
            .map(|read| {
                read.into_records()
                    .into_iter()
                    .map(|record| record.target)
                    .collect::<Vec<_>>()
            })
    };
    if exact_targets(approval.transition_relation, transition, 1).as_deref() != Some(&[*entity])
        || exact_targets(approval.proposal_relation, *entity, 1).as_deref()
            != Some(&[approval.proposal])
    {
        return None;
    }
    let mut expected_evidence = approval.evidence.to_vec();
    expected_evidence.sort_unstable();
    let mut actual_evidence =
        exact_targets(approval.evidence_relation, *entity, expected_evidence.len())?;
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
