use crate::runtime::WorthUiPlanSwapReceipt;

#[cfg(any(test, feature = "certification-support"))]
pub(crate) struct WorthUiQueryAwarePlanSwap {
    plan_swap: Box<WorthUiPlanSwapReceipt>,
    query_retirement: worth_ui_query_binding::WorthUiOperationLiveRetirement,
    plan_decision: crate::runtime::WorthUiExecutablePlanDecision,
    catalog_successor_receipt: Option<crate::runtime::UiAllocationCatalogSuccessorReceipt>,
}

#[cfg(any(test, feature = "certification-support"))]
pub(crate) enum WorthUiQueryAwarePlanOutcome {
    SemanticNoOp,
    Activated(Box<WorthUiQueryAwarePlanSwap>),
}

pub(crate) struct WorthUiPreparedQueryAwarePlanSwap {
    activation: super::committed_allocation_attempt::UiPreparedCommittedAllocationActivation,
    plan_decision: crate::runtime::WorthUiExecutablePlanDecision,
    catalog_successor_receipt: Option<crate::runtime::UiAllocationCatalogSuccessorReceipt>,
}

pub(crate) struct WorthUiPreparedApplicationPlanSwap {
    activation: Box<WorthUiPreparedQueryAwarePlanSwap>,
    catalog_successor_receipt: crate::runtime::UiAllocationCatalogSuccessorReceipt,
}

pub(crate) struct WorthUiApplicationPlanSwap {
    plan_swap: Box<WorthUiPlanSwapReceipt>,
    query_retirement: worth_ui_query_binding::WorthUiOperationLiveRetirement,
    plan_decision: crate::runtime::WorthUiExecutablePlanDecision,
    catalog_successor_receipt: crate::runtime::UiAllocationCatalogSuccessorReceipt,
}

pub(crate) enum WorthUiPreparedQueryAwarePlanOutcome {
    SemanticNoOp(Box<crate::runtime::WorthUiSemanticNoOpReceipt>),
    Activation(Box<WorthUiPreparedQueryAwarePlanSwap>),
}

#[cfg(any(test, feature = "certification-support"))]
impl WorthUiQueryAwarePlanOutcome {
    pub(super) fn into_plan_swap_after_asserting_no_query_retirement(
        self,
    ) -> WorthUiPlanSwapReceipt {
        match self {
            Self::Activated(publication) => {
                publication.into_plan_swap_after_asserting_no_query_retirement()
            }
            Self::SemanticNoOp => {
                panic!("lower-level publication helper received a semantic no-op")
            }
        }
    }
}

#[cfg(any(test, feature = "certification-support"))]
impl WorthUiQueryAwarePlanSwap {
    pub(super) fn new(
        plan_swap: Box<WorthUiPlanSwapReceipt>,
        query_retirement: worth_ui_query_binding::WorthUiOperationLiveRetirement,
        plan_decision: crate::runtime::WorthUiExecutablePlanDecision,
        catalog_successor_receipt: Option<crate::runtime::UiAllocationCatalogSuccessorReceipt>,
    ) -> Self {
        Self {
            plan_swap,
            query_retirement,
            plan_decision,
            catalog_successor_receipt,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Box<WorthUiPlanSwapReceipt>,
        worth_ui_query_binding::WorthUiOperationLiveRetirement,
        crate::runtime::WorthUiExecutablePlanDecision,
        Option<crate::runtime::UiAllocationCatalogSuccessorReceipt>,
    ) {
        (
            self.plan_swap,
            self.query_retirement,
            self.plan_decision,
            self.catalog_successor_receipt,
        )
    }

    #[cfg(any(test, feature = "certification-support"))]
    fn into_plan_swap_after_asserting_no_query_retirement(self) -> WorthUiPlanSwapReceipt {
        let (plan_swap, query_retirement, plan_decision, catalog_successor_receipt) =
            self.into_parts();
        assert!(
            query_retirement.is_empty(),
            "lower-level activation helpers cannot erase Query retirement; use the public Query-aware cutover"
        );
        debug_assert_ne!(
            plan_decision.kind(),
            crate::runtime::WorthUiExecutablePlanDecisionKind::Denied
        );
        debug_assert!(catalog_successor_receipt.is_none());
        *plan_swap
    }
}

impl WorthUiPreparedQueryAwarePlanOutcome {
    pub(crate) fn into_activation(
        self,
    ) -> Result<
        Box<WorthUiPreparedQueryAwarePlanSwap>,
        Box<crate::runtime::WorthUiSemanticNoOpReceipt>,
    > {
        match self {
            Self::SemanticNoOp(receipt) => Err(receipt),
            Self::Activation(activation) => Ok(activation),
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn commit_once(
        self,
        runtime: &mut crate::runtime::WorthUiRuntime,
        active_app: Option<&mut crate::facade::WorthUiApp>,
    ) -> WorthUiQueryAwarePlanOutcome {
        match self {
            Self::SemanticNoOp(_) => WorthUiQueryAwarePlanOutcome::SemanticNoOp,
            Self::Activation(prepared) => WorthUiQueryAwarePlanOutcome::Activated(Box::new(
                prepared.commit_once(runtime, active_app),
            )),
        }
    }
}

impl WorthUiPreparedQueryAwarePlanSwap {
    pub(super) fn new(
        activation: super::committed_allocation_attempt::UiPreparedCommittedAllocationActivation,
        plan_decision: crate::runtime::WorthUiExecutablePlanDecision,
        catalog_successor_receipt: Option<crate::runtime::UiAllocationCatalogSuccessorReceipt>,
    ) -> Self {
        Self {
            activation,
            plan_decision,
            catalog_successor_receipt,
        }
    }

    pub(crate) fn into_application_activation(
        mut self: Box<Self>,
    ) -> Result<WorthUiPreparedApplicationPlanSwap, Box<Self>> {
        let Some(catalog_successor_receipt) = self.catalog_successor_receipt.take() else {
            return Err(self);
        };
        Ok(WorthUiPreparedApplicationPlanSwap {
            activation: self,
            catalog_successor_receipt,
        })
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn commit_once(
        self,
        runtime: &mut crate::runtime::WorthUiRuntime,
        active_app: Option<&mut crate::facade::WorthUiApp>,
    ) -> WorthUiQueryAwarePlanSwap {
        let publication = self.activation.commit_once(runtime, active_app);
        let (plan_swap, query_retirement, derived_index_counters) = publication.into_parts();
        let mut catalog_successor_receipt = self.catalog_successor_receipt;
        if let Some(receipt) = catalog_successor_receipt.as_mut() {
            receipt.bind_derived_index_work(derived_index_counters);
        }
        WorthUiQueryAwarePlanSwap::new(
            plan_swap,
            query_retirement,
            self.plan_decision,
            catalog_successor_receipt,
        )
    }
}

impl WorthUiPreparedApplicationPlanSwap {
    pub(crate) fn candidate_replacement_authority(
        &self,
    ) -> Option<&crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority>
    {
        self.activation.activation.candidate_replacement_authority()
    }

    pub(crate) fn candidate_plan(&self) -> &crate::runtime::WorthUiActiveExecutionPlan {
        self.activation.activation.candidate_plan()
    }

    pub(crate) fn candidate_query_binding(
        &self,
    ) -> &worth_ui_query_binding::WorthUiRuntimeQueryBinding {
        self.activation.activation.candidate_query_binding()
    }

    pub(crate) fn candidate_allocation_catalog(
        &self,
    ) -> crate::runtime::UiMountedAllocationProjectionCatalog {
        self.activation.activation.candidate_allocation_catalog()
    }

    pub(crate) fn candidate_plan_digest(&self) -> u64 {
        self.activation.activation.candidate_plan_digest()
    }

    pub(crate) fn candidate_allocation_truth_revision(&self) -> u64 {
        self.activation
            .activation
            .candidate_allocation_truth_revision()
    }

    pub(crate) fn candidate_runtime_observation(
        &self,
    ) -> crate::runtime::WorthUiActiveRuntimeObservation {
        self.activation.activation.candidate_runtime_observation()
    }

    pub(crate) fn candidate_scheduler_state(
        &self,
    ) -> crate::runtime::UiAllocationFrameDispatcherState {
        self.activation.activation.candidate_scheduler_state()
    }

    pub(crate) fn previous_active_plan_digest(&self) -> u64 {
        self.activation.activation.previous_active_plan_digest()
    }

    pub(crate) fn plan_decision(&self) -> crate::runtime::WorthUiExecutablePlanDecision {
        self.activation.plan_decision
    }

    pub(crate) fn commit_once(
        self,
        runtime: &mut crate::runtime::WorthUiRuntime,
        active_app: &mut crate::facade::WorthUiApp,
    ) -> WorthUiApplicationPlanSwap {
        let WorthUiPreparedQueryAwarePlanSwap {
            activation,
            plan_decision,
            catalog_successor_receipt,
        } = *self.activation;
        debug_assert!(catalog_successor_receipt.is_none());
        let publication = activation.commit_once(runtime, Some(active_app));
        let (plan_swap, query_retirement, derived_index_counters) = publication.into_parts();
        let mut catalog_successor_receipt = self.catalog_successor_receipt;
        catalog_successor_receipt.bind_derived_index_work(derived_index_counters);
        WorthUiApplicationPlanSwap {
            plan_swap,
            query_retirement,
            plan_decision,
            catalog_successor_receipt,
        }
    }
}

impl WorthUiApplicationPlanSwap {
    pub(crate) fn into_parts(
        self,
    ) -> (
        Box<WorthUiPlanSwapReceipt>,
        worth_ui_query_binding::WorthUiOperationLiveRetirement,
        crate::runtime::WorthUiExecutablePlanDecision,
        crate::runtime::UiAllocationCatalogSuccessorReceipt,
    ) {
        (
            self.plan_swap,
            self.query_retirement,
            self.plan_decision,
            self.catalog_successor_receipt,
        )
    }
}
