use std::sync::Arc;

use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

use super::{
    WorthQueryProviderExecutionPlanContract, WorthQueryProviderExecutionPlanView,
    WorthQueryProviderPlanReadmission, WorthQueryProviderProductAffinity,
    WorthQueryProviderSessionAffinity, WorthQueryProviderSessionDenialKind,
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolCounters,
    WorthQueryProviderSessionProtocolStage, WorthQueryProviderSessionRecoveryPosture,
    WorthQueryProviderSessionTokenAdmission,
};

pub(super) struct WorthQueryProviderPlanReadmissionSeal(());

impl WorthQueryProviderPlanReadmissionSeal {
    fn new() -> Self {
        Self(())
    }
}
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

pub(super) enum WorthQueryProviderRunBorrow<'run> {
    Direct(&'run mut WorthQueryRunningDirectRun),
    Workflow(&'run mut WorthQueryRunningWorkflowRun),
}

impl WorthQueryProviderRunBorrow<'_> {
    pub(super) fn run_identity(&self) -> &str {
        match self {
            Self::Direct(run) => run.identity(),
            Self::Workflow(run) => run.identity(),
        }
    }
}

pub struct WorthQueryAdmittedProviderExecutionPlan<'run> {
    run: WorthQueryProviderRunBorrow<'run>,
    contract: WorthQueryProviderExecutionPlanContract,
    provider: Arc<WorthQueryGraphProviderAnchor>,
    product: WorthQueryProviderProductAffinity,
    counters: WorthQueryProviderSessionProtocolCounters,
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
        let mut counters = WorthQueryProviderSessionProtocolCounters::default();
        let operation = run.provider_plan_operation();
        let session = run.provider_plan_session();
        let (resources, evidence) = run.provider_plan_resources();
        counters.checked_authority();
        validate_common_authority(
            WorthQueryProviderPlanAuthorityObservation {
                operation,
                session_identity: session.identity(),
                session_attempt_identity: session.attempt_identity(),
                resource_authority_matches: operation
                    .admits_provider_plan_resources(None, resources),
                evidence_session_identity: evidence.provider_session_identity(),
                evidence_attempt_identity: evidence.provider_session_attempt_identity(),
                bridge: run.provider_plan_bridge_basis(),
                graph,
                stage_identity: None,
            },
            &counters,
        )?;
        let provider = retain_session_provider(graph, &counters)?;
        let snapshot_identity = run.execution_snapshot_reference();
        let contract = operation
            .provider_plan_contract(WorthQueryValidatedProviderPlan {
                operation,
                stage_identity: None,
                managed_run_identity: run.identity(),
                execution_basis_identity: run.provider_plan_bridge_basis().identity().as_str(),
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
            run: WorthQueryProviderRunBorrow::Direct(run),
            contract,
            provider,
            product: WorthQueryProviderProductAffinity::standalone(),
            counters,
        })
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
            run: WorthQueryProviderRunBorrow::Workflow(run),
            contract,
            provider,
            product: WorthQueryProviderProductAffinity::standalone(),
            counters,
        })
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
        let Some(expected_product) = self.contract.application_product_observation() else {
            return Err(failure(
                WorthQueryProviderSessionDenialKind::ForeignExecutionBasis,
                "application product cannot bind a standalone provider plan",
                &self.counters,
            ));
        };
        if product.observation() != expected_product {
            return Err(failure(
                WorthQueryProviderSessionDenialKind::ForeignExecutionBasis,
                "provider plan snapshot does not belong to the selected product occurrence",
                &self.counters,
            ));
        }
        self.product = WorthQueryProviderProductAffinity::application(product);
        Ok(self)
    }

    pub fn readmit(
        mut self,
    ) -> Result<WorthQueryProviderPlanReadmission<'run>, WorthQueryProviderSessionFailure> {
        self.counters.called_provider();
        let admission = WorthQueryProviderSessionTokenAdmission::new(&self.contract);
        let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.provider.readmit_session(
                &WorthQueryProviderExecutionPlanView::new(&self.contract),
                admission,
            )
        }));
        let token = match invocation {
            Ok(Ok(token)) => token,
            Ok(Err(failure)) => {
                return Err(failure.at_stage(
                    WorthQueryProviderSessionProtocolStage::PlanReadmission,
                    self.counters,
                ));
            }
            Err(_) => {
                return Err(WorthQueryProviderSessionFailure::new(
                    WorthQueryProviderSessionDenialKind::ProviderPanicked,
                    WorthQueryProviderSessionProtocolStage::PlanReadmission,
                    "provider panicked while readmitting the sealed execution plan",
                    self.counters,
                )
                .with_recovery_posture(
                    WorthQueryProviderSessionRecoveryPosture::RecoveryRequired,
                ));
            }
        };
        if !token.belongs_to(&self.contract) {
            return Err(WorthQueryProviderSessionFailure::new(
                WorthQueryProviderSessionDenialKind::TokenNotMintedForPlan,
                WorthQueryProviderSessionProtocolStage::PlanReadmission,
                "provider returned a token minted for another plan or generation",
                self.counters,
            )
            .with_recovery_posture(WorthQueryProviderSessionRecoveryPosture::RecoveryRequired));
        }
        self.counters.minted_token();
        Ok(WorthQueryProviderPlanReadmission::from_admitted(
            WorthQueryProviderSessionAffinity::mint(
                self.run,
                self.contract,
                self.product,
                self.provider,
                token,
            ),
            self.counters,
            WorthQueryProviderPlanReadmissionSeal::new(),
        ))
    }
}

mod validation;
use validation::{failure, retain_session_provider, undeclared_scope, validate_common_authority};
