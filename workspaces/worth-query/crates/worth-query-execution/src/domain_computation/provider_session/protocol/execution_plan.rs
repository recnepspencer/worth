use std::sync::Arc;

use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

use super::{
    WorthQueryProviderExecutionPlanContract, WorthQueryProviderExecutionPlanView,
    WorthQueryProviderProductAffinity, WorthQueryProviderSessionAffinity,
    WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolCounters, WorthQueryProviderSessionProtocolStage,
    WorthQueryProviderSessionRecoveryPosture, WorthQueryProviderSessionTokenAdmission,
};

use crate::domain_computation::managed_run::{
    WorthQueryRunningDirectRun, WorthQueryRunningWorkflowRun,
};
use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;

struct WorthQueryProviderPlanAuthorityObservation<'a> {
    operation: &'a crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    session_identity: &'a str,
    session_attempt_identity: &'a str,
    resource_authority_matches: bool,
    evidence_session_identity: &'a str,
    evidence_attempt_identity: &'a str,
    bridge: &'a worth_runtime_bridge::facade::BridgeBoundExecutionBasis,
    graph: &'a WorthQueryInstalledGraphParticipationAuthority,
    stage_identity: Option<&'a str>,
}

pub(crate) struct WorthQueryValidatedProviderPlan<'a> {
    operation: &'a crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    stage_identity: Option<&'a str>,
    managed_run_identity: &'a str,
    execution_basis_identity: &'a str,
    admitted_session_identity: &'a str,
    resource_attempt_identity: &'a str,
    graph: &'a WorthQueryInstalledGraphParticipationAuthority,
    snapshot_identity: &'a str,
    resource_envelope_identity: &'a str,
    provider_identity: &'a str,
    provider_generation: u64,
}

impl WorthQueryValidatedProviderPlan<'_> {
    pub(crate) fn belongs_to(
        &self,
        operation: &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    ) -> bool {
        std::ptr::eq(self.operation, operation)
    }
    pub(crate) const fn operation(
        &self,
    ) -> &crate::domain_computation::WorthQueryExecutionBoundOperationAuthority {
        self.operation
    }
    pub(crate) const fn stage_identity(&self) -> Option<&str> {
        self.stage_identity
    }
    pub(super) const fn managed_run_identity(&self) -> &str {
        self.managed_run_identity
    }
    pub(super) const fn execution_basis_identity(&self) -> &str {
        self.execution_basis_identity
    }
    pub(super) const fn admitted_session_identity(&self) -> &str {
        self.admitted_session_identity
    }
    pub(super) const fn resource_attempt_identity(&self) -> &str {
        self.resource_attempt_identity
    }
    pub(super) const fn graph(&self) -> &WorthQueryInstalledGraphParticipationAuthority {
        self.graph
    }
    pub(super) const fn snapshot_identity(&self) -> &str {
        self.snapshot_identity
    }
    pub(super) const fn resource_envelope_identity(&self) -> &str {
        self.resource_envelope_identity
    }
    pub(super) const fn provider_identity(&self) -> &str {
        self.provider_identity
    }
    pub(super) const fn provider_generation(&self) -> u64 {
        self.provider_generation
    }
}

mod owned_direct;
pub(in crate::domain_computation) use owned_direct::{
    CoupledProviderPlanAdmission, ReturnedProviderDirectRun,
};
mod direct;

pub(super) enum WorthQueryProviderRunCustody<'run> {
    Direct(&'run mut WorthQueryRunningDirectRun),
    Workflow(&'run mut WorthQueryRunningWorkflowRun),
    CoupledDirect(Box<owned_direct::OwnedDirectRun>),
}

impl WorthQueryProviderRunCustody<'_> {
    pub(super) fn run_identity(&self) -> &str {
        match self {
            Self::Direct(run) => run.identity(),
            Self::Workflow(run) => run.identity(),
            Self::CoupledDirect(run) => run.run().identity(),
        }
    }
}

pub struct WorthQueryAdmittedProviderExecutionPlan<'run> {
    run: WorthQueryProviderRunCustody<'run>,
    pub(super) contract: WorthQueryProviderExecutionPlanContract,
    provider: Arc<WorthQueryGraphProviderAnchor>,
    product: WorthQueryProviderProductAffinity,
    pub(super) counters: WorthQueryProviderSessionProtocolCounters,
}

impl std::fmt::Debug for WorthQueryAdmittedProviderExecutionPlan<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryAdmittedProviderExecutionPlan")
            .field("identity", &self.contract.identity())
            .field("run_identity", &self.run.run_identity())
            .finish_non_exhaustive()
    }
}

impl<'run> WorthQueryAdmittedProviderExecutionPlan<'run> {
    pub(crate) fn direct(
        run: &'run mut WorthQueryRunningDirectRun,
        graph: &WorthQueryInstalledGraphParticipationAuthority,
    ) -> Result<Self, WorthQueryProviderSessionFailure> {
        let (contract, provider, counters) = direct::prepare(run, graph)?;
        Ok(Self {
            run: WorthQueryProviderRunCustody::Direct(run),
            contract,
            provider,
            product: WorthQueryProviderProductAffinity::standalone(),
            counters,
        })
    }

    pub(in crate::domain_computation) fn coupled_direct(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        run: WorthQueryRunningDirectRun,
        graph: &WorthQueryInstalledGraphParticipationAuthority,
    ) -> CoupledProviderPlanAdmission<'run> {
        let (run, returned) = match owned_direct::OwnedDirectRun::new(execution, run) {
            Ok(owned) => owned,
            Err((run, cause)) => {
                return CoupledProviderPlanAdmission::AllocationDenied { run, cause }
            }
        };
        let prepared =
            direct::prepare(run.run(), graph).map(|(contract, provider, counters)| Self {
                run: WorthQueryProviderRunCustody::CoupledDirect(run),
                contract,
                provider,
                product: WorthQueryProviderProductAffinity::standalone(),
                counters,
            });
        match prepared {
            Ok(plan) => CoupledProviderPlanAdmission::Admitted { plan, returned },
            Err(cause) => CoupledProviderPlanAdmission::Refused { cause, returned },
        }
    }

    pub(in crate::domain_computation) fn workflow_stage(
        run: &'run mut WorthQueryRunningWorkflowRun,
        stage_identity: &str,
        graph: &WorthQueryInstalledGraphParticipationAuthority,
        owner: &crate::domain_computation::managed_run::WorthQueryWorkflowProviderPlanPermit,
    ) -> Result<Self, WorthQueryProviderSessionFailure> {
        let mut counters = WorthQueryProviderSessionProtocolCounters::default();
        let (resources, evidence) = run
            .provider_plan_stage_resources(stage_identity, owner)
            .ok_or_else(|| undeclared_scope(&counters))?;
        let operation = run.provider_plan_operation(owner);
        let session = run.provider_plan_session(owner);
        counters.checked_authority();
        validate_common_authority(
            WorthQueryProviderPlanAuthorityObservation {
                operation,
                session_identity: session.identity(),
                session_attempt_identity: session.attempt_identity(),
                resource_authority_matches: operation
                    .admits_provider_plan_resources(Some(stage_identity), &resources),
                evidence_session_identity: evidence.provider_session_identity(),
                evidence_attempt_identity: evidence.provider_session_attempt_identity(),
                bridge: run.provider_plan_bridge_basis(owner),
                graph,
                stage_identity: Some(stage_identity),
            },
            &counters,
        )?;
        let provider = retain_session_provider(graph, &counters)?;
        let snapshot_identity = run.execution_snapshot_reference();
        let contract = operation
            .provider_plan_contract(WorthQueryValidatedProviderPlan {
                operation,
                stage_identity: Some(stage_identity),
                managed_run_identity: run.identity(),
                execution_basis_identity: run.provider_plan_bridge_basis(owner).identity().as_str(),
                admitted_session_identity: session.identity(),
                resource_attempt_identity: session.attempt_identity(),
                graph,
                snapshot_identity: &snapshot_identity,
                resource_envelope_identity: resources.envelope_identity(),
                provider_identity: provider.provider_identity(),
                provider_generation: provider.provider_generation(),
            })
            .ok_or_else(|| undeclared_scope(&counters))?;
        counters.bound_closure_items(contract.closure_width());
        Ok(Self {
            run: WorthQueryProviderRunCustody::Workflow(run),
            contract,
            provider,
            product: WorthQueryProviderProductAffinity::standalone(),
            counters,
        })
    }

    pub(in crate::domain_computation::provider_session::protocol) fn provider_grant_control(
        &self,
    ) -> &Arc<super::ProviderSessionGrantControl> {
        &self.provider.session_grant_control
    }

    pub(in crate::domain_computation::provider_session::protocol) fn preparation_provider(
        &self,
    ) -> Arc<WorthQueryGraphProviderAnchor> {
        Arc::clone(&self.provider)
    }

    pub fn identity(&self) -> &str {
        self.contract.identity()
    }

    pub fn contract(&self) -> &WorthQueryProviderExecutionPlanContract {
        &self.contract
    }

    pub fn counters(&self) -> WorthQueryProviderSessionProtocolCounters {
        self.counters
    }

    pub(in crate::domain_computation) fn bind_application_product(
        mut self,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Self, WorthQueryProviderSessionFailure> {
        self.require_current_home(Some(product.observation().owner_identity()))?;
        let Some(expected_product) = self.contract.application_product_identity() else {
            return Err(failure(
                WorthQueryProviderSessionDenialKind::ForeignExecutionBasis,
                "application product cannot bind a standalone provider plan",
                &self.counters,
            ));
        };
        if &crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
            product.observation(),
        ) != expected_product
        {
            return Err(failure(
                WorthQueryProviderSessionDenialKind::ForeignExecutionBasis,
                "provider plan snapshot does not belong to the selected product occurrence",
                &self.counters,
            ));
        }
        self.product = WorthQueryProviderProductAffinity::application(product);
        Ok(self)
    }

    /// Issue a portable, one-use grant without opening a physical session.
    /// The separate receiver owns the run, product and provider ports.
    pub fn admit_session(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'run>,
    ) -> Result<
        (
            super::WorthQueryProviderSessionGrant,
            super::WorthQueryProviderSessionReceiver<'run>,
        ),
        WorthQueryProviderSessionFailure,
    > {
        self.require_current_home(phase.world_owner())?;
        self.request_for_session(phase)?;
        let home_owner = match &self.product {
            WorthQueryProviderProductAffinity::Standalone => match &self.run {
                WorthQueryProviderRunCustody::Direct(run) => run.execution_owner_identity(),
                WorthQueryProviderRunCustody::CoupledDirect(owned) => {
                    owned.run().execution_owner_identity()
                }
                WorthQueryProviderRunCustody::Workflow(run) => run.execution_owner_identity(),
            },
            WorthQueryProviderProductAffinity::Application(product) => {
                Some(product.observation().owner_identity())
            }
        };
        let admission = WorthQueryProviderSessionTokenAdmission::new(&self.contract, home_owner)?;
        Ok(super::readmission::issue(self, admission))
    }
}

mod incarnation;
mod readmission;
mod validation;
use validation::{failure, retain_session_provider, undeclared_scope, validate_common_authority};

mod court_doors_for_test;
