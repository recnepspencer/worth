use super::{
    workflow_stage_lineage::lineage_traversal, WorthQueryBoundGraphExecutionReceipt,
    WorthQueryWorkflowEffectEvidence, WorthQueryWorkflowPredecessorReceipt,
    WorthQueryWorkflowStageEffectDenial, WorthQueryWorkflowStageExecutionAuthority,
    WorthQueryWorkflowStageExecutionScope, WorthQueryWorkflowStageExecutorFailure,
    WorthQueryWorkflowStageLineageDenial, WorthQueryWorkflowStageWorkspace,
};

pub struct WorthQueryWorkflowStageExecutionContext<'a> {
    pub(super) operation_identity: &'a str,
    pub(super) binding_identity: &'a str,
    pub(super) run_identity: &'a str,
    pub(super) stage: &'a worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    predecessor_receipts: Vec<WorthQueryWorkflowPredecessorReceipt<'a>>,
    effect_workflow_binding: crate::workflow::WorkflowContextBinding,
    basis: crate::basis_lifecycle::BasisFamily,
    installed_read: Option<&'a crate::ordinary::read::WorthQueryReadDeclaration>,
    operation_graph_reads:
        &'a [worth_query_installation::facade::WorthQueryDomainOperationGraphReadRole],
    graph_receipts: &'a [WorthQueryBoundGraphExecutionReceipt],
    resources: &'a super::WorthQueryAdmittedExecutionResourcePlan,
    resource_evidence: &'a super::WorthQueryExecutionResourceAttemptEvidence,
    provider_session_identity: &'a str,
    query_authority: crate::identity_authority::QueryCanonicalAuthority,
    pub(super) identity_evolution_basis_identity: String,
    pub(super) artifact_access_authority:
        Option<std::sync::Arc<crate::domain_installation::WorthQueryArtifactAccessAuthority>>,
    pub(super) artifact_production_authority:
        Option<std::sync::Arc<crate::domain_installation::WorthQueryArtifactProductionAuthority>>,
}

impl<'a> WorthQueryWorkflowStageExecutionContext<'a> {
    pub(crate) fn new(
        scope: WorthQueryWorkflowStageExecutionScope<'a>,
        authority: WorthQueryWorkflowStageExecutionAuthority<'a>,
    ) -> Self {
        Self {
            operation_identity: scope.operation_identity,
            binding_identity: scope.binding_identity,
            run_identity: scope.run_identity,
            stage: scope.stage,
            predecessor_receipts: scope
                .predecessor_receipts
                .iter()
                .map(|receipt| WorthQueryWorkflowPredecessorReceipt::new(receipt))
                .collect(),
            effect_workflow_binding: authority.effect_workflow_binding,
            basis: authority.basis,
            installed_read: authority.installed_read,
            operation_graph_reads: authority.operation_graph_reads,
            graph_receipts: authority.graph_receipts,
            resources: authority.resources,
            resource_evidence: authority.resource_evidence,
            provider_session_identity: authority.provider_session_identity,
            query_authority: authority.query_authority,
            identity_evolution_basis_identity: authority.identity_evolution_basis_identity,
            artifact_access_authority: authority.artifact_access_authority,
            artifact_production_authority: authority.artifact_production_authority,
        }
    }

    pub(crate) fn artifact_production_authority(
        &self,
    ) -> Option<std::sync::Arc<crate::domain_installation::WorthQueryArtifactProductionAuthority>>
    {
        self.artifact_production_authority
            .as_ref()
            .map(std::sync::Arc::clone)
    }

    pub(crate) fn artifact_access_authority(
        &self,
    ) -> Option<std::sync::Arc<crate::domain_installation::WorthQueryArtifactAccessAuthority>> {
        self.artifact_access_authority
            .as_ref()
            .map(std::sync::Arc::clone)
    }

    pub fn operation_identity(&self) -> &str {
        self.operation_identity
    }
    pub fn binding_identity(&self) -> &str {
        self.binding_identity
    }
    pub fn run_identity(&self) -> &str {
        self.run_identity
    }
    pub fn resources(&self) -> &super::WorthQueryAdmittedExecutionResourcePlan {
        self.resources
    }
    pub fn resource_evidence(&self) -> &super::WorthQueryExecutionResourceAttemptEvidence {
        self.resource_evidence
    }
    pub fn provider_session_identity(&self) -> &str {
        self.provider_session_identity
    }
    pub fn stage(&self) -> &worth_query_installation::facade::WorthQueryPortableWorkflowStage {
        self.stage
    }
    pub fn predecessor_receipts(&self) -> &[WorthQueryWorkflowPredecessorReceipt<'a>] {
        &self.predecessor_receipts
    }
    pub fn execute_identity_evolution(
        &self,
        establishing_effect: &WorthQueryWorkflowEffectEvidence,
    ) -> Result<
        crate::identity_evolution::InstalledIdentityEvolutionOutcome,
        WorthQueryWorkflowStageLineageDenial,
    > {
        let mutation_receipt = establishing_effect
            .mutation_receipt()
            .ok_or(WorthQueryWorkflowStageLineageDenial::RuntimeMutationEvidenceRequired)?;
        let (descriptor, lifecycle_target) = lineage_traversal(mutation_receipt)?;
        let query =
            crate::identity_evolution::IdentityEvolutionQueryContext::installed_operation_lineage(
                &self.query_authority,
                self.operation_identity,
                &self.identity_evolution_basis_identity,
                descriptor,
            );
        let admitted = crate::identity_evolution::admit_identity_evolution_query(query)
            .map_err(WorthQueryWorkflowStageLineageDenial::IdentityEvolutionAdmission)?;
        let artifact =
            crate::identity_evolution::execute_admitted_identity_evolution_query(&admitted)
                .map_err(WorthQueryWorkflowStageLineageDenial::IdentityEvolutionAdmission)?;
        crate::identity_evolution::InstalledIdentityEvolutionOutcome::from_execution(
            artifact,
            mutation_receipt.continuity_mutation_evidence().cloned(),
            lifecycle_target,
            crate::identity_evolution::InstalledIdentityEvolutionBinding {
                operation_identity: self.operation_identity,
                run_identity: self.run_identity,
                stage_identity: self.stage.identity(),
                effect_receipt_identity: establishing_effect.receipt_identity().to_owned(),
                establishing_entity_targets: mutation_receipt
                    .deltas()
                    .iter()
                    .map(|delta| delta.entity_identity().clone())
                    .collect(),
            },
        )
        .ok_or(WorthQueryWorkflowStageLineageDenial::IdentityEvolutionOutcomeMismatch)
    }

    pub fn execute_identity_correspondence(
        &self,
        establishing_effect: &WorthQueryWorkflowEffectEvidence,
        observation: super::WorthQueryInstalledCorrespondenceObservation,
    ) -> Result<
        crate::identity_evolution::InstalledIdentityEvolutionOutcome,
        WorthQueryWorkflowStageLineageDenial,
    > {
        let mutation_receipt = establishing_effect
            .mutation_receipt()
            .ok_or(WorthQueryWorkflowStageLineageDenial::RuntimeMutationEvidenceRequired)?;
        let query =
            crate::identity_evolution::IdentityEvolutionQueryContext::installed_operation_correspondence(
                &self.query_authority,
                self.operation_identity,
                &self.identity_evolution_basis_identity,
                observation.into_engine_comparison(),
            );
        let admitted = crate::identity_evolution::admit_identity_evolution_query(query)
            .map_err(WorthQueryWorkflowStageLineageDenial::IdentityEvolutionAdmission)?;
        let artifact =
            crate::identity_evolution::execute_admitted_identity_evolution_query(&admitted)
                .map_err(WorthQueryWorkflowStageLineageDenial::IdentityEvolutionAdmission)?;
        crate::identity_evolution::InstalledIdentityEvolutionOutcome::from_execution(
            artifact,
            None,
            None,
            crate::identity_evolution::InstalledIdentityEvolutionBinding {
                operation_identity: self.operation_identity,
                run_identity: self.run_identity,
                stage_identity: self.stage.identity(),
                effect_receipt_identity: establishing_effect.receipt_identity().to_owned(),
                establishing_entity_targets: mutation_receipt
                    .deltas()
                    .iter()
                    .map(|delta| delta.entity_identity().clone())
                    .collect(),
            },
        )
        .ok_or(WorthQueryWorkflowStageLineageDenial::IdentityEvolutionOutcomeMismatch)
    }

    pub fn graph_projection(
        &self,
        role: &str,
    ) -> Option<&super::WorthQueryExecutionGraphReadProduct> {
        if !self
            .stage
            .semantics()
            .graph_read_roles
            .iter()
            .any(|declared| declared == role)
        {
            return None;
        }
        self.graph_receipts
            .iter()
            .find(|receipt| {
                receipt.role() == role
                    && receipt.kind()
                        == crate::domain_installation::WorthQueryGraphProviderCallKind::Project
            })
            .and_then(WorthQueryBoundGraphExecutionReceipt::graph_read_product)
    }

    pub fn execute_mutation(
        &self,
        command: crate::runtime::WorthQueryWriteCommand,
        workspace: &mut WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<WorthQueryWorkflowEffectEvidence, WorthQueryWorkflowStageEffectDenial> {
        if !self
            .stage
            .semantics()
            .effect_roles
            .contains(&worth_query_installation::facade::WorthQueryOperationEffectFamily::Mutation)
        {
            return Err(WorthQueryWorkflowStageEffectDenial::UndeclaredEffectFamily);
        }
        let execution = workspace
            .workspace
            .execute_ordinary_authoritative_mutation(command, false)
            .map_err(|error| WorthQueryWorkflowStageEffectDenial::Runtime(format!("{error:?}")))?;
        let evidence = WorthQueryWorkflowEffectEvidence::runtime_mutation(
            execution.into_receipt(),
            &self.effect_workflow_binding,
            self.basis,
        );
        workspace.executed_effects.push(evidence.clone());
        Ok(evidence)
    }

    pub fn execute_installed_read(
        &self,
        role: &str,
        workspace: &mut WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<
        crate::ordinary::read::WorthQueryReadCompletion,
        WorthQueryWorkflowStageExecutorFailure,
    > {
        let role_is_admitted = self
            .stage
            .semantics()
            .graph_read_roles
            .iter()
            .any(|declared| declared == role)
            && self.operation_graph_reads.iter().any(|declared| {
                declared.role == role
                    && declared.participation
                        == worth_query_installation::facade::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph
            });
        if !role_is_admitted {
            return Err(WorthQueryWorkflowStageExecutorFailure::new(
                crate::domain_installation::WorthQueryOperationFailureClass::Indeterminate,
                "workflow stage lacks the installed primary read role",
            ));
        }
        let declaration = self.installed_read.ok_or_else(|| {
            WorthQueryWorkflowStageExecutorFailure::new(
                crate::domain_installation::WorthQueryOperationFailureClass::Indeterminate,
                "workflow operation has no Query-installed read declaration",
            )
        })?;
        if workspace.installed_read_executions != 0 {
            return Err(WorthQueryWorkflowStageExecutorFailure::new(
                crate::domain_installation::WorthQueryOperationFailureClass::Indeterminate,
                "installed canonical read may execute only once per workflow stage",
            ));
        }
        workspace.installed_read_executions += 1;
        declaration
            .clone_for_installed_execution()
            .using(crate::ordinary::read::current())
            .run(workspace.workspace)
            .into_result()
            .map_err(|stop| {
                WorthQueryWorkflowStageExecutorFailure::new(
                    crate::domain_installation::WorthQueryOperationFailureClass::Dependency,
                    format!("{stop:?}"),
                )
            })
    }

    pub(crate) fn requires_primary_read(&self) -> bool {
        self.operation_graph_reads.iter().any(|declared| {
            self.stage
                .semantics()
                .graph_read_roles
                .contains(&declared.role)
                && declared.participation
                    == worth_query_installation::facade::WorthQueryOperationGraphParticipation::PrimaryLogicalGraph
        })
    }
}
