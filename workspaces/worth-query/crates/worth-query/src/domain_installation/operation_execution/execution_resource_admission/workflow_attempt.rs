use std::collections::BTreeMap;

use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, operation_phase_basis, WorthQueryOperationPhaseProof,
    WorthQueryResourceAdmittedOperationPhase,
};
use worth_proof::TransitionOutcome;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest;

use super::{
    admit_execution_resource_plan, WorthQueryAdmittedWorkflowResourcePlan,
    WorthQueryExecutionResourceAdmissionCounters, WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenialKind as Kind,
    WorthQueryWorkflowExecutionResourceAttempt,
};

pub type WorthQueryWorkflowResourceAdmissionOutcome<D, O, F, L> = TransitionOutcome<
    WorthQueryAdmittedWorkflowOperation<D, O, F, L>,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
>;

pub struct WorthQueryAdmittedWorkflowOperation<D, O, F, L: BasisOperationLane> {
    pub(crate) bound: crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    pub(crate) graph: std::sync::Arc<super::super::WorthQueryInstalledWorkflowGraph>,
    pub(crate) executor: std::sync::Arc<super::super::WorthQueryInstalledWorkflowStageExecutor>,
    pub(crate) parallel_posture:
        crate::domain_installation::operating_world::WorthQueryBoundWorkflowParallelPosture,
    pub(crate) resource_attempt: WorthQueryWorkflowExecutionResourceAttempt,
    pub(crate) phase_proof: WorthQueryOperationPhaseProof<WorthQueryResourceAdmittedOperationPhase>,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryAdmittedWorkflowOperation<D, O, F, L> {
    pub fn resources(&self) -> &WorthQueryAdmittedWorkflowResourcePlan {
        self.resource_attempt.resources()
    }

    pub fn provider_session_identity(&self) -> &str {
        self.resource_attempt.evidence().provider_session_identity()
    }

    pub fn provider_session_attempt_identity(&self) -> &str {
        self.resource_attempt
            .evidence()
            .provider_session_attempt_identity()
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>
where
    O: crate::domain_installation::WorthQueryExecutableDomainOperation<
        D,
        F,
        Execution = crate::domain_installation::WorthQueryWorkflowOperation,
    >,
{
    pub fn admit_workflow_resources(
        self,
        request: WorthQueryExecutionResourceRequest,
        workspace: &crate::runtime::WorthQueryWorkspace,
    ) -> WorthQueryWorkflowResourceAdmissionOutcome<D, O, F, L> {
        let counters = WorthQueryExecutionResourceAdmissionCounters {
            runtime_authority_checks: 1,
            ..Default::default()
        };
        let witness =
            crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness::from_authority(
                std::sync::Arc::clone(self.operation().domain_authority()),
            );
        if let Err(denial) = workspace.validate_installed_domain_witness::<D>(&witness) {
            let kind = denial.kind();
            let denial = WorthQueryExecutionResourceAdmissionDenial::new(
                Kind::RuntimeAuthority(kind),
                format!("{denial:?}"),
                counters,
            );
            return runtime_denial(kind, denial);
        }
        let (graph, executor, parallel_posture) = self.workflow_providers();
        let graph = std::sync::Arc::clone(graph);
        let executor = std::sync::Arc::clone(executor);
        let parallel_posture = parallel_posture.clone();
        let support = self
            .execution_authority()
            .workflow_operation_support()
            .expect("bound workflow execution authority carries installed support")
            .clone();
        let operation = match admit_execution_resource_plan(
            self.binding_identity(),
            &self.definition().semantics().resources,
            &request,
            support.clone(),
            counters,
        ) {
            Ok(plan) => plan,
            Err(denial) => return admission_denial_outcome(denial),
        };
        let stages = match lower_stages(
            &self,
            &request,
            WorthQueryExecutionResourceAdmissionCounters::default(),
        ) {
            Ok(stages) => stages,
            Err(denial) => return admission_denial_outcome(denial),
        };
        let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(operation, stages);
        let resource_attempt = match workspace
            .query_execution_runtime()
            .start_workflow_resource_attempt(self.execution_authority(), resources)
        {
            Ok(attempt) => attempt,
            Err(denial) => return admission_denial_outcome(denial),
        };
        let mut basis = operation_phase_basis(self.authority_proof()).clone();
        basis.resource_admission_identity =
            Some(resource_attempt.resources().identity().to_owned());
        let phase_proof = mint_operation_phase_proof(
            resource_attempt.resources().identity().to_owned(),
            Some(self.authority_proof().payload().identity()),
            basis,
        );
        TransitionOutcome::Success(WorthQueryAdmittedWorkflowOperation {
            bound: self,
            graph,
            executor,
            parallel_posture,
            resource_attempt,
            phase_proof,
        })
    }
}

fn lower_stages<D, O, F, L: BasisOperationLane>(
    bound: &crate::domain_installation::WorthQueryBoundDomainOperation<D, O, F, L>,
    request: &WorthQueryExecutionResourceRequest,
    counters: WorthQueryExecutionResourceAdmissionCounters,
) -> Result<
    BTreeMap<String, super::WorthQueryAdmittedExecutionResourcePlan>,
    WorthQueryExecutionResourceAdmissionDenial,
> {
    let workflow = match &bound.definition().semantics().workflow {
        worth_query_installation::facade::WorthQueryOperationWorkflowContract::Declared(
            workflow,
        ) => workflow,
        worth_query_installation::facade::WorthQueryOperationWorkflowContract::NotRequired => {
            return Err(WorthQueryExecutionResourceAdmissionDenial::new(
                Kind::ResourceContract,
                "workflow resource admission requires an installed workflow definition",
                counters,
            ))
        }
    };
    workflow
        .stages()
        .iter()
        .map(|stage| {
            let support = bound
                .execution_authority()
                .workflow_stage_support(stage.identity())
                .expect("bound workflow authority carries every installed stage support")
                .clone();
            admit_execution_resource_plan(
                &format!("{}:{}", bound.binding_identity(), stage.identity()),
                &stage.semantics().resources,
                request,
                support,
                counters,
            )
            .map(|plan| (stage.identity().to_owned(), plan))
        })
        .collect()
}

fn runtime_denial<T>(
    kind: crate::domain_installation::WorthQueryDomainHandleDenialKind,
    denial: WorthQueryExecutionResourceAdmissionDenial,
) -> TransitionOutcome<
    T,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
> {
    use crate::domain_installation::WorthQueryDomainHandleDenialKind as Runtime;
    match kind {
        Runtime::StaleInstallationGeneration => TransitionOutcome::Stale(denial),
        Runtime::PackageIdentityChanged => TransitionOutcome::RebindRequired(denial),
        Runtime::DomainNotInstalled | Runtime::ForeignRuntime => TransitionOutcome::Denied(denial),
    }
}

fn admission_denial_outcome<T>(
    denial: WorthQueryExecutionResourceAdmissionDenial,
) -> TransitionOutcome<
    T,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
    WorthQueryExecutionResourceAdmissionDenial,
> {
    match denial.kind() {
        Kind::Backpressured | Kind::AsyncExecutionRequired => TransitionOutcome::Deferred(denial),
        _ => TransitionOutcome::Denied(denial),
    }
}
