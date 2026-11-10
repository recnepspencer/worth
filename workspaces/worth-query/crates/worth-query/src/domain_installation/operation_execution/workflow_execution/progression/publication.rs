use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryConsumedOperationPhase,
    WorthQueryOperationPhaseProof, WorthQueryPublishedOperationPhase,
    WorthQuerySettledOperationPhase,
};
use crate::domain_installation::{
    WorthQueryBoundProjectionRequest, WorthQueryCompletedWorkflowTrace,
    WorthQueryConsumerProjectionContract, WorthQueryExecutableDomainOperation,
    WorthQueryNativeAccessDenial, WorthQueryNativeAccessKey, WorthQueryNativeAccessLayout,
    WorthQueryNativeFieldAccess, WorthQueryPublishingOperation, WorthQueryWorkflowRunCounters,
    WorthQueryWorkflowValue,
};
use crate::identity::hash_parts;
use crate::ordinary::read::{WorthQueryProjectionDeclaration, WorthQueryProjectionOutcome};
use crate::projection_consumption::{
    ProjectionConsumptionWarnings, WorthQueryConsumedProjectionAuthority,
};
use worth_proof::TransitionOutcome;

impl<D, O, F, L: BasisOperationLane> WorthQueryCompletedWorkflowTrace<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<D, F, Publication = WorthQueryPublishingOperation>,
{
    pub fn publish(
        self,
    ) -> TransitionOutcome<
        WorthQueryPublishedWorkflow<D, O, F, L>,
        WorthQueryWorkflowPublicationDenial,
        std::convert::Infallible,
        WorthQueryWorkflowPublicationDenial,
        WorthQueryWorkflowPublicationDenial,
        WorthQueryWorkflowPublicationDenial,
    > {
        if !self.run.bound.installation_is_current() {
            return TransitionOutcome::Stale(
                WorthQueryWorkflowPublicationDenial::StaleInstallationGeneration,
            );
        }
        let canonical = &self.run.bound.definition().semantics().canonical_query;
        let Some(publication_stage) =
            self.run.receipts.iter().find(|receipt| {
                self.run.graph.stages().iter().any(|stage| {
                    stage.identity() == receipt.stage_identity() && stage.is_publishable()
                }) && matches!(receipt.output(), WorthQueryWorkflowValue::Projection(_))
            })
        else {
            return TransitionOutcome::Denied(
                WorthQueryWorkflowPublicationDenial::PublishableStageMissing,
            );
        };
        let WorthQueryWorkflowValue::Projection(completion) = publication_stage.output() else {
            return TransitionOutcome::Failed(
                WorthQueryWorkflowPublicationDenial::PublicationStageOutputMismatch,
            );
        };
        if !completion.validates_installed_publication(
            canonical,
            self.run.bound.basis().normalized().family(),
            publication_stage.execution_snapshot(),
            self.run
                .bound
                .operation()
                .domain_authority()
                .runtime_authority(),
        ) {
            return TransitionOutcome::Denied(
                WorthQueryWorkflowPublicationDenial::ProjectionMismatch,
            );
        }
        let publication_stage_identity = publication_stage.stage_identity().to_string();
        let publication_subjects_by_evidence_identity = completion
            .result()
            .rows()
            .iter()
            .map(|row| {
                (
                    row.identity().evidence_identity().as_str().to_owned(),
                    row.identity().clone(),
                )
            })
            .collect();
        let receipt_identity = hash_parts(&[
            "worth_query_workflow_publication_v1".into(),
            format!("trace:{}", self.identity),
            format!("stage:{}", publication_stage.identity()),
        ]);
        let phase_proof = mint_operation_phase_proof(
            receipt_identity.clone(),
            Some(self.phase_proof.payload().identity()),
            operation_phase_basis(&self.phase_proof).clone(),
        );
        TransitionOutcome::Success(WorthQueryPublishedWorkflow {
            trace: self,
            publication_stage_identity,
            publication_subjects_by_evidence_identity,
            receipt_identity,
            phase_proof,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowPublicationDenial {
    StaleInstallationGeneration,
    PublishableStageMissing,
    PublicationStageOutputMismatch,
    ProjectionMismatch,
}

pub struct WorthQueryPublishedWorkflow<D, O, F, L: BasisOperationLane> {
    trace: WorthQueryCompletedWorkflowTrace<D, O, F, L>,
    publication_stage_identity: String,
    publication_subjects_by_evidence_identity:
        std::collections::HashMap<String, crate::memory_workspace::WorthQueryEntityIdentity>,
    receipt_identity: String,
    phase_proof: WorthQueryOperationPhaseProof<WorthQueryPublishedOperationPhase>,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryPublishedWorkflow<D, O, F, L> {
    pub fn receipt_identity(&self) -> &str {
        &self.receipt_identity
    }
    pub fn publication_stage_identity(&self) -> &str {
        &self.publication_stage_identity
    }
    pub fn trace(&self) -> &WorthQueryCompletedWorkflowTrace<D, O, F, L> {
        &self.trace
    }
    pub(crate) fn publication_carries_entity(
        &self,
        subject: &crate::memory_workspace::WorthQueryEntityIdentity,
    ) -> bool {
        self.publication_subjects_by_evidence_identity
            .get(subject.evidence_identity().as_str())
            == Some(subject)
    }
    pub fn consume(
        mut self,
        consumer: WorthQueryConsumerProjectionContract<D, O, F, L>,
        request: WorthQueryProjectionDeclaration,
    ) -> TransitionOutcome<
        WorthQueryConsumedWorkflowProjection<D, O, F, L>,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
    > {
        let bound = &self.trace.run.bound;
        if !bound.installation_is_current() {
            return TransitionOutcome::Stale(
                WorthQueryWorkflowConsumptionDenial::StaleInstallationGeneration,
            );
        }
        if !consumer.binds_capability(bound.capability_identity())
            || consumer.binding_identity() != bound.binding_identity()
            || consumer.operation_identity() != bound.definition().identity()
            || consumer.installation_generation() != bound.operation().installation_generation()
            || consumer.basis_identity() != bound.basis().capability_digest()
        {
            return TransitionOutcome::Denied(
                WorthQueryWorkflowConsumptionDenial::ConsumerContractMismatch,
            );
        }
        let Some(stage) = self
            .trace
            .run
            .receipts
            .iter()
            .find(|receipt| receipt.stage_identity() == self.publication_stage_identity)
        else {
            return TransitionOutcome::Failed(
                WorthQueryWorkflowConsumptionDenial::PublicationStageMissing,
            );
        };
        let WorthQueryWorkflowValue::Projection(completion) = stage.output() else {
            return TransitionOutcome::Failed(
                WorthQueryWorkflowConsumptionDenial::PublicationStageOutputMismatch,
            );
        };
        self.trace.run.counters.consumption_contacts += 1;
        let (authority, warnings) = match completion.consume_projection(request).into_admitted() {
            Ok(admitted) => admitted,
            Err(outcome @ WorthQueryProjectionOutcome::Violation(_)) => {
                return TransitionOutcome::Denied(WorthQueryWorkflowConsumptionDenial::Projection(
                    Box::new(outcome),
                ));
            }
            Err(outcome @ WorthQueryProjectionOutcome::Deferred(_)) => {
                return TransitionOutcome::Deferred(
                    WorthQueryWorkflowConsumptionDenial::Projection(Box::new(outcome)),
                );
            }
            Err(outcome @ WorthQueryProjectionOutcome::Unavailable(_)) => {
                return TransitionOutcome::Failed(WorthQueryWorkflowConsumptionDenial::Projection(
                    Box::new(outcome),
                ));
            }
            Err(outcome) => {
                return TransitionOutcome::Failed(WorthQueryWorkflowConsumptionDenial::Projection(
                    Box::new(outcome),
                ));
            }
        };
        let consumption_identity = hash_parts(&[
            "worth_query_consumed_workflow_v1".into(),
            format!("publication:{}", self.receipt_identity),
            format!("consumption:{}", authority.receipt().receipt_digest()),
        ]);
        let phase_proof = mint_operation_phase_proof(
            consumption_identity,
            Some(self.phase_proof.payload().identity()),
            operation_phase_basis(&self.phase_proof).clone(),
        );
        TransitionOutcome::Success(WorthQueryConsumedWorkflowProjection {
            published: self,
            consumer,
            authority,
            warnings,
            native_access: None,
            phase_proof,
        })
    }

    pub fn consume_bound(
        self,
        request: WorthQueryBoundProjectionRequest<D, O, F, L>,
    ) -> TransitionOutcome<
        WorthQueryConsumedWorkflowProjection<D, O, F, L>,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
    > {
        let (consumer, declaration, plan) = request.into_parts();
        match self.consume(consumer, declaration) {
            TransitionOutcome::Success(mut consumed) => {
                match WorthQueryNativeAccessLayout::admit(
                    plan,
                    &consumed.consumer,
                    &consumed.authority,
                ) {
                    Ok(layout) => {
                        consumed.native_access = Some(layout);
                        TransitionOutcome::Success(consumed)
                    }
                    Err(denial) => TransitionOutcome::Denied(
                        WorthQueryWorkflowConsumptionDenial::NativeAccess(denial),
                    ),
                }
            }
            TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
            TransitionOutcome::Deferred(denial) => TransitionOutcome::Deferred(denial),
            TransitionOutcome::Stale(denial) => TransitionOutcome::Stale(denial),
            TransitionOutcome::RebindRequired(denial) => TransitionOutcome::RebindRequired(denial),
            TransitionOutcome::Failed(denial) => TransitionOutcome::Failed(denial),
        }
    }
}

#[derive(Debug)]
pub enum WorthQueryWorkflowConsumptionDenial {
    StaleInstallationGeneration,
    ConsumerContractMismatch,
    NativeAccess(WorthQueryNativeAccessDenial),
    PublicationStageMissing,
    PublicationStageOutputMismatch,
    Projection(Box<WorthQueryProjectionOutcome>),
}

pub struct WorthQueryConsumedWorkflowProjection<D, O, F, L: BasisOperationLane> {
    published: WorthQueryPublishedWorkflow<D, O, F, L>,
    consumer: WorthQueryConsumerProjectionContract<D, O, F, L>,
    authority: Box<WorthQueryConsumedProjectionAuthority>,
    warnings: Option<ProjectionConsumptionWarnings>,
    native_access: Option<WorthQueryNativeAccessLayout>,
    phase_proof: WorthQueryOperationPhaseProof<WorthQueryConsumedOperationPhase>,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryConsumedWorkflowProjection<D, O, F, L> {
    pub fn authority(&self) -> &WorthQueryConsumedProjectionAuthority {
        &self.authority
    }
    pub fn warnings(&self) -> Option<&ProjectionConsumptionWarnings> {
        self.warnings.as_ref()
    }
    pub fn trace(&self) -> &WorthQueryCompletedWorkflowTrace<D, O, F, L> {
        &self.published.trace
    }

    pub fn settle(
        self,
    ) -> TransitionOutcome<
        WorthQuerySettledWorkflowProjection<D, O, F, L>,
        WorthQueryWorkflowConsumptionDenial,
        std::convert::Infallible,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
        WorthQueryWorkflowConsumptionDenial,
    > {
        if !self.published.trace.run.bound.installation_is_current() {
            return TransitionOutcome::Stale(
                WorthQueryWorkflowConsumptionDenial::StaleInstallationGeneration,
            );
        }
        let identity = hash_parts(&[
            "worth_query_installed_workflow_settlement_v1".into(),
            format!("publication:{}", self.published.receipt_identity),
            format!("consumption:{}", self.authority.receipt().receipt_digest()),
        ]);
        let phase_proof = mint_operation_phase_proof(
            identity.clone(),
            Some(self.phase_proof.payload().identity()),
            operation_phase_basis(&self.phase_proof).clone(),
        );
        TransitionOutcome::Success(WorthQuerySettledWorkflowProjection {
            consumed: self,
            identity,
            phase_proof,
        })
    }
}

pub struct WorthQuerySettledWorkflowProjection<D, O, F, L: BasisOperationLane> {
    consumed: WorthQueryConsumedWorkflowProjection<D, O, F, L>,
    identity: String,
    phase_proof: WorthQueryOperationPhaseProof<WorthQuerySettledOperationPhase>,
}

impl<D, O, F, L: BasisOperationLane> WorthQuerySettledWorkflowProjection<D, O, F, L> {
    pub(crate) fn bound_operation(
        &self,
    ) -> &crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L> {
        self.trace().bound()
    }
    pub(crate) fn consumer_contract(&self) -> &WorthQueryConsumerProjectionContract<D, O, F, L> {
        &self.consumed.consumer
    }
    pub(crate) fn publication_stage_identity(&self) -> &str {
        self.consumed.published.publication_stage_identity()
    }
    pub(crate) fn workflow_run_identity(&self) -> &str {
        self.trace().workflow_run_identity()
    }
    pub(crate) fn installed_workflow_read(
        &self,
    ) -> Option<&crate::ordinary::read::WorthQueryReadDeclaration> {
        self.trace().installed_workflow_read()
    }
    pub(crate) fn native_access_layout(&self) -> Option<&WorthQueryNativeAccessLayout> {
        self.consumed.native_access.as_ref()
    }
    pub fn identity(&self) -> &str {
        debug_assert_eq!(self.phase_proof.payload().identity(), self.identity);
        debug_assert_eq!(
            self.phase_proof.payload().predecessor_identity(),
            Some(self.consumed.phase_proof.payload().identity())
        );
        &self.identity
    }
    pub fn authority(&self) -> &WorthQueryConsumedProjectionAuthority {
        self.consumed.authority()
    }
    pub fn trace(&self) -> &WorthQueryCompletedWorkflowTrace<D, O, F, L> {
        self.consumed.trace()
    }
    pub fn warnings(&self) -> Option<&ProjectionConsumptionWarnings> {
        self.consumed.warnings()
    }
    pub fn counters(&self) -> WorthQueryWorkflowRunCounters {
        self.trace().counters()
    }

    pub fn native_value<'a>(
        &'a self,
        key: &WorthQueryNativeAccessKey,
        row: usize,
    ) -> Result<WorthQueryNativeFieldAccess<'a>, WorthQueryNativeAccessDenial> {
        let Some(layout) = &self.consumed.native_access else {
            return Err(WorthQueryNativeAccessLayout::unbound_denial(
                &self.consumed.authority,
                key,
            ));
        };
        layout.access(&self.consumed.authority, key, row)
    }

    pub fn native_access_binding_counters(
        &self,
    ) -> Option<crate::domain_installation::WorthQueryNativeAccessBindingCounters> {
        self.consumed
            .native_access
            .as_ref()
            .map(WorthQueryNativeAccessLayout::binding_counters)
    }
}
