use crate::domain_computation::primary_graph::PrimaryGraphHandleWorkflowExt;
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenialCommitExt as _;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::super::{
    PublishedWorkflowInstanceRef, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationIdempotencyResolution, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryApplicationUncommitted,
};
use crate::domain_computation::primary_graph::workflow::proposal::derive_workflow_proposal_context_identity;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_proposal_program::PreparedWorkflowProposal;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_proposal_program::publication::reference;
pub use reference::PublishedWorkflowProposalRef;
pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_proposal_program::publication::read_only;
/// A workflow proposal that landed: the proposal, the transition it opened, its commit
/// receipt from this actual publication.
#[derive(Debug)]
pub struct PerformedWorkflowProposal {
    proposal: PublishedWorkflowProposalRef,
    transition: EntityId,
    receipt: super::super::WorthQueryApplicationCommitReceipt,
}

impl PerformedWorkflowProposal {
    pub const fn proposal(&self) -> &PublishedWorkflowProposalRef {
        &self.proposal
    }
    pub const fn transition(&self) -> EntityId {
        self.transition
    }
    pub fn node_path(&self) -> &str {
        self.proposal.node_path()
    }
    pub const fn receipt(&self) -> &super::super::WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
}

/// What executing a workflow proposal produced. `ProjectionDenied` means the commit landed
/// but its receipt did not record this proposal on the instance's branch.
#[derive(Debug)]
pub enum WorkflowProposalOutcome {
    RecoveredCommitted(crate::domain_computation::primary_graph::provider::WorthQueryRecoveredApplicationCommitReceipt),
    Published(PerformedWorkflowProposal),
    /// The commit did not land. A fresh landed commit is the
    /// performed variant.
    Application(WorthQueryApplicationUncommitted),
    ProjectionDenied(super::super::WorthQueryApplicationCommitReceipt),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_workflow_proposal_replay<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        instance: PublishedWorkflowInstanceRef,
    ) -> Result<Option<WorkflowProposalOutcome>, WorthQueryApplicationIdempotencyResolutionDenial>
    where
        Input: Clone + Send + Sync + 'static,
    {
        let context_identity = derive_workflow_proposal_context_identity(instance.entity_id());
        let resolution = self
            .resolve_admitted_application_idempotency_with_request(
                execution,
                admission,
                idempotency.bind_workflow_proposal_context(&context_identity),
            )?
            .into_resolution();
        let outcome = match resolution {
            WorthQueryApplicationIdempotencyResolution::RecoveredCommitted(receipt) => {
                WorkflowProposalOutcome::RecoveredCommitted(receipt)
            }
            WorthQueryApplicationIdempotencyResolution::ReleasedOriginal(_) => WorkflowProposalOutcome::Application(WorthQueryApplicationUncommitted::Denied(crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::original_effect_released())),
            WorthQueryApplicationIdempotencyResolution::Unseen => return Ok(None),
            WorthQueryApplicationIdempotencyResolution::ProductUnpublished(owner) => {
                WorkflowProposalOutcome::Application(
                    WorthQueryApplicationUncommitted::ProductUnpublished(owner),
                )
            }
            WorthQueryApplicationIdempotencyResolution::ProductPreparationPending(owner) => {
                WorkflowProposalOutcome::Application(
                    WorthQueryApplicationUncommitted::ProductPreparationPending(owner),
                )
            }
            WorthQueryApplicationIdempotencyResolution::IntentDrift => {
                WorkflowProposalOutcome::Application(WorthQueryApplicationUncommitted::Denied(
                    super::super::WorthQueryApplicationCommitDenial::idempotency_intent_drift(),
                ))
            }
        };
        Ok(Some(outcome))
    }

    pub fn compare_and_commit_workflow_proposal<Operation: 'static, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,

        prepared: PreparedWorkflowProposal<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowProposalOutcome
    where
        Input: Clone + Send + Sync + 'static,
    {
        if let Err(cause) = phase.execution_request_for(&self.runtime.product_runtime) {
            return WorkflowProposalOutcome::Application(
                crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(cause)
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit"),
            );
        }
        let PreparedWorkflowProposal {
            program,
            program_revision,
            transition_identity_locator,
            proposal_identity_locator,
            proposal_node_path_locator,
            proposal_context_identity,
            instance,
            operation,
            input_type,
            input_identity,
            source_identity,
            captured_identity,
            progress_update,
        } = prepared;
        let presented = match self
            .installed_program_support()
            .ok_or_else(
                super::super::WorthQueryApplicationCommitDenial::application_program_required,
            )
            .and_then(|support| support.present_for_commit(&program_revision))
        {
            Ok(presented) => presented,
            Err(denial) => {
                return WorkflowProposalOutcome::Application(
                    WorthQueryApplicationUncommitted::Denied(denial),
                );
            }
        };
        let outcome = self.compare_and_commit_application_for_program_action(
            execution,
            phase,
            &presented,
            program,
            idempotency
                .bind_workflow_proposal_context(&proposal_context_identity)
                .bind_workflow_captured_inputs(captured_identity),
            crate::domain_computation::application_aftermath::ApplicationCommitCausality::Ordinary,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        let context = ProjectionContext {
            transition_identity_locator,
            proposal_identity_locator,
            proposal_node_path_locator,
            instance,
            operation,
            input_type,
            input_identity,
            source_identity,
        };
        let projected = match outcome.landed() {
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Live { receipt }) => project(receipt, context),
            Ok(crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationLandedCommit::Recovered(receipt)) => return WorkflowProposalOutcome::RecoveredCommitted(receipt),
            Err(uncommitted) => WorkflowProposalOutcome::Application(uncommitted),
        };
        if let (Some(progress_update), WorkflowProposalOutcome::Published(performed)) =
            (progress_update, &projected)
        {
            let committed_revision = performed.receipt().commit_reference().version_id;
            let progress_key = progress_update.key();
            let _ = self
                .runtime
                .primary_provider
                .graph_ref()
                .with_workflow_instance_progress_mut(progress_key, |retention| {
                    progress_update.apply(
                        retention,
                        committed_revision,
                        performed.transition(),
                        None,
                    )
                });
        }
        projected
    }
}

struct ProjectionContext {
    transition_identity_locator: AspectFieldLocator,
    proposal_identity_locator: AspectFieldLocator,
    proposal_node_path_locator: AspectFieldLocator,
    instance: PublishedWorkflowInstanceRef,
    operation: String,
    input_type: String,
    input_identity: [u8; 32],
    source_identity: Option<[u8; 32]>,
}

fn project(
    receipt: super::super::WorthQueryApplicationCommitReceipt,
    context: ProjectionContext,
) -> WorkflowProposalOutcome {
    if receipt.product_branch() != context.instance.branch() {
        return WorkflowProposalOutcome::ProjectionDenied(receipt);
    }
    let transition = unique_created_text(&receipt, &context.transition_identity_locator);
    let proposal = unique_created_text_pair(
        &receipt,
        &context.proposal_identity_locator,
        &context.proposal_node_path_locator,
    );
    let (Some((transition, _)), Some((proposal, identity, node_path))) = (transition, proposal)
    else {
        return WorkflowProposalOutcome::ProjectionDenied(receipt);
    };
    let Some(published) = PublishedWorkflowProposalRef::from_committed_proposal(
        &receipt,
        &context.instance,
        proposal,
        identity.into(),
        context.operation.into(),
        context.input_type.into(),
        context.input_identity,
        context.source_identity,
        node_path.into(),
    ) else {
        return WorkflowProposalOutcome::ProjectionDenied(receipt);
    };
    WorkflowProposalOutcome::Published(PerformedWorkflowProposal {
        proposal: published,
        transition,
        receipt,
    })
}

fn unique_created_text(
    receipt: &super::super::WorthQueryApplicationCommitReceipt,
    locator: &AspectFieldLocator,
) -> Option<(EntityId, String)> {
    let candidates = receipt
        .committed_changes()
        .entity_changes()
        .filter(|(_, change)| {
            *change == worth_relational::facade::publication::RecordStructuralChange::Created
        })
        .filter_map(|(entity, _)| {
            receipt
                .committed_changes()
                .committed_field_values(entity, &[locator])
                .and_then(|values| match values.as_slice() {
                    [AspectValue::String(InternedString::Stored(value))] => {
                        let value = value.as_string();
                        {
                            Some((entity, value.clone()))
                        }
                    }
                    [AspectValue::String(InternedString::Raw(value))] => {
                        Some((entity, value.clone()))
                    }
                    _ => None,
                })
        })
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(candidate.clone())
}

fn unique_created_text_pair(
    receipt: &super::super::WorthQueryApplicationCommitReceipt,
    first: &AspectFieldLocator,
    second: &AspectFieldLocator,
) -> Option<(EntityId, String, String)> {
    let candidates = receipt
        .committed_changes()
        .entity_changes()
        .filter(|(_, change)| {
            *change == worth_relational::facade::publication::RecordStructuralChange::Created
        })
        .filter_map(|(entity, _)| {
            receipt
                .committed_changes()
                .committed_field_values(entity, &[first, second])
                .and_then(|values| match values.as_slice() {
                    [a, b] => Some((entity, a.raw_text()?.to_owned(), b.raw_text()?.to_owned())),
                    _ => None,
                })
        })
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        return None;
    };
    Some(candidate.clone())
}
