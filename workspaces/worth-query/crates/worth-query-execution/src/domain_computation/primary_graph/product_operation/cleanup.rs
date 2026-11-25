//! Application-owned completion of product cleanup across Query and Bridge.

use worth_query_execution_runtime::facade::ports::ProductBranchOwnerCleanupInternalPort as _;
use worth_runtime_bridge::facade::BridgeConditionalDenial;

use crate::domain_computation::execution_runtime::product_world::{
    OwnerCleanupRetirements, WorthQueryProductBranchOwnerCleanup,
    WorthQueryProductBranchOwnerCleanupDenial, WorthQueryProductBranchOwnerCleanupReceipt,
    WorthQueryProductBranchOwnerCleanupWork,
};

#[derive(Debug)]
pub enum WorthQueryApplicationProductBranchCleanupDenial {
    ObligationDischarge(
        crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
    ),
    SignalBasisObservation(worth_signal::facade::branch::SignalBranchBasisObservationDenial),
    ConditionalRetirement(BridgeConditionalDenial),
    Product(WorthQueryProductBranchOwnerCleanupDenial),
}

/// Retains every owner needed to finish one product cleanup obligation.
#[must_use = "product cleanup must be completed or retained for retry"]
pub struct WorthQueryApplicationProductBranchCleanup {
    product: WorthQueryProductBranchOwnerCleanup,
    conditional: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
    obligations: Option<
        std::sync::Arc<
            crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
        >,
    >,
}

impl std::fmt::Debug for WorthQueryApplicationProductBranchCleanup {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationProductBranchCleanup")
            .field("pending", &self.product.pending_work())
            .finish_non_exhaustive()
    }
}

impl WorthQueryApplicationProductBranchCleanup {
    pub(in crate::domain_computation) fn new(
        product: WorthQueryProductBranchOwnerCleanup,
        conditional: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
        obligations: Option<
            std::sync::Arc<
                crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
            >,
        >,
    ) -> Self {
        Self {
            product,
            conditional,
            obligations,
        }
    }

    pub fn pending_work(&self) -> Vec<WorthQueryProductBranchOwnerCleanupWork> {
        self.product.pending_work()
    }

    pub fn retry(
        self,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryApplicationProductBranchCleanupFailure,
    > {
        let mut retirements = OwnerCleanupRetirements::default();
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        self.retry_with_retirements(
            &worth_execution::ExecutionRequest::serial(&host),
            &mut retirements,
        )
    }

    /// Retries every cleanup owner under this runtime's original admitted request.
    /// A foreign or interrupted phase returns the same cleanup without effects.
    pub fn retry_in_advancement(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryApplicationProductBranchCleanupFailure,
    > {
        let mut retirements = OwnerCleanupRetirements::default();
        let execution = match self.product.execution_for(phase) {
            Ok(execution) => execution,
            Err(denial) => {
                return Err(Self::failed(
                    self.product,
                    self.conditional,
                    self.obligations,
                    WorthQueryApplicationProductBranchCleanupDenial::Product(
                        WorthQueryProductBranchOwnerCleanupDenial::Execution(denial),
                    ),
                ));
            }
        };
        self.retry_with_retirements(&execution, &mut retirements)
    }

    pub(crate) fn retry_with_retirements(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        retirements: &mut OwnerCleanupRetirements,
    ) -> Result<
        WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryApplicationProductBranchCleanupFailure,
    > {
        let Self {
            product,
            conditional,
            obligations,
        } = self;
        if let Some((history, occurrence)) = obligations
            .as_ref()
            .zip(product.retired_application_occurrence())
        {
            let branch_name = occurrence.branch().name().as_str();
            let lifecycle = occurrence.incarnation().ordinal();
            for position in history.active_output_demands_for_branch(branch_name, lifecycle) {
                if let Err(denial) = history.discharge_output_demand(execution, position) {
                    return Err(Self::failed(
                        product,
                        conditional,
                        obligations,
                        WorthQueryApplicationProductBranchCleanupDenial::ObligationDischarge(
                            denial,
                        ),
                    ));
                }
            }
            for commit in history.active_output_sources_for_branch(branch_name, lifecycle) {
                if let Err(denial) = history.discharge_output_source(execution, commit) {
                    return Err(Self::failed(
                        product,
                        conditional,
                        obligations,
                        WorthQueryApplicationProductBranchCleanupDenial::ObligationDischarge(
                            denial,
                        ),
                    ));
                }
            }
        }
        if let Err(denial) = product.release_retired_history_with_request(execution, retirements) {
            return Err(Self::failed(
                product,
                conditional,
                obligations,
                WorthQueryApplicationProductBranchCleanupDenial::Product(denial),
            ));
        }
        let bases = match product.pending_signal_bases() {
            Ok(bases) => bases,
            Err(denial) => {
                return Err(Self::failed(
                    product,
                    conditional,
                    obligations,
                    WorthQueryApplicationProductBranchCleanupDenial::SignalBasisObservation(denial),
                ));
            }
        };
        for basis in bases {
            let result = match &conditional {
                crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner::Memory(owner) => owner.write().unwrap_or_else(std::sync::PoisonError::into_inner).retire_owned_conditionals_on_signal_branch(&basis),
                crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner::Authored(_)
                | crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner::Stopped => Err(BridgeConditionalDenial::operation_route_refused("individual authored branch retirement requires its durable disposition")),
            };
            if let Err(denial) = result {
                return Err(Self::failed(
                    product,
                    conditional,
                    obligations,
                    WorthQueryApplicationProductBranchCleanupDenial::ConditionalRetirement(denial),
                ));
            }
        }
        match product.retry_in_request_with_retirements(execution, retirements) {
            Ok(receipt) => Ok(receipt),
            Err(failure) => {
                let denial = failure.denial();
                Err(Self::failed(
                    failure.into_cleanup(),
                    conditional,
                    obligations,
                    WorthQueryApplicationProductBranchCleanupDenial::Product(denial),
                ))
            }
        }
    }

    fn failed(
        product: WorthQueryProductBranchOwnerCleanup,
        conditional: crate::domain_computation::primary_graph::managed_bridge::conditional_owner::ConditionalOwner,
        obligations: Option<
            std::sync::Arc<
                crate::domain_computation::primary_graph::query_obligations::history::QueryObligationHistory,
            >,
        >,
        denial: WorthQueryApplicationProductBranchCleanupDenial,
    ) -> WorthQueryApplicationProductBranchCleanupFailure {
        WorthQueryApplicationProductBranchCleanupFailure {
            denial,
            cleanup: Self {
                product,
                conditional,
                obligations,
            },
        }
    }
}

#[derive(Debug)]
pub struct WorthQueryApplicationProductBranchCleanupFailure {
    denial: WorthQueryApplicationProductBranchCleanupDenial,
    cleanup: WorthQueryApplicationProductBranchCleanup,
}

impl WorthQueryApplicationProductBranchCleanupFailure {
    pub const fn denial(&self) -> &WorthQueryApplicationProductBranchCleanupDenial {
        &self.denial
    }

    pub fn into_cleanup(self) -> WorthQueryApplicationProductBranchCleanup {
        self.cleanup
    }
}
