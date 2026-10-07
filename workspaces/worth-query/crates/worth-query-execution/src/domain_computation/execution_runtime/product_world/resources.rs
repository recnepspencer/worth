use worth_runtime_world::facade::RuntimeWorldBudgets;

use super::WorthQueryProductWorldClock;

/// Exhaustive live resources for one Query-owned Product World.
///
/// This value is supplied by the caller and consumed at World construction.
/// Owner services continue to come from the authoritative Relational and
/// sealed Bridge installations.
#[derive(Clone)]
pub struct WorthQueryProductWorldResources {
    budgets: RuntimeWorldBudgets,
    clock: WorthQueryProductWorldClock,
    invalidation: super::super::WorthQueryInvalidationResources,
    execution_authority: Option<std::sync::Arc<worth_execution::ExecutionAuthority>>,
}

impl WorthQueryProductWorldResources {
    pub fn install(
        budgets: worth_runtime_world::facade::RuntimeWorldBudgetInstallation,
        clock: WorthQueryProductWorldClock,
        invalidation: super::super::WorthQueryInvalidationResources,
    ) -> Result<Self, worth_runtime_world::facade::RuntimeWorldBudgetDenial> {
        RuntimeWorldBudgets::install(budgets).map(|budgets| Self::new(budgets, clock, invalidation))
    }

    pub fn new(
        budgets: RuntimeWorldBudgets,
        clock: WorthQueryProductWorldClock,
        invalidation: super::super::WorthQueryInvalidationResources,
    ) -> Self {
        Self {
            budgets,
            clock,
            invalidation,
            execution_authority: None,
        }
    }

    /// Supplies the host's existing process authority to this World. Cloned
    /// resources share that authority; Query constructs no replacement pool.
    pub fn with_execution_authority(
        mut self,
        authority: std::sync::Arc<worth_execution::ExecutionAuthority>,
    ) -> Self {
        self.execution_authority = Some(authority);
        self
    }

    pub fn invalidation_resources(&self) -> super::super::WorthQueryInvalidationResources {
        self.invalidation.clone()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RuntimeWorldBudgets,
        WorthQueryProductWorldClock,
        super::super::WorthQueryInvalidationResources,
        Option<std::sync::Arc<worth_execution::ExecutionAuthority>>,
    ) {
        (
            self.budgets,
            self.clock,
            self.invalidation,
            self.execution_authority,
        )
    }

    #[cfg(test)]
    pub(crate) const fn budgets(&self) -> &RuntimeWorldBudgets {
        &self.budgets
    }
}

#[cfg(test)]
pub(in crate::domain_computation) fn test_product_world_resources(
) -> WorthQueryProductWorldResources {
    test_product_world_resources_with_history_limit(1_024)
}

#[cfg(any(test, feature = "test-primary-graph-faults"))]
pub(crate) fn test_product_world_resources_with_history_limit(
    retained_composite_commits: u64,
) -> WorthQueryProductWorldResources {
    use worth_runtime_world::facade::{
        RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
        RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
        RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
        RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
    };

    WorthQueryProductWorldResources::new(
        RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 128,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits,
                history_metadata_bytes: 16 * 1024 * 1024,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 512,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 128,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 128,
                retained_partial_metadata_bytes: 16 * 1024 * 1024,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 1_024,
                in_flight_pin_acquisition_reservations: 256,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 256,
            },
        })
        .expect("the test Product World budget is valid"),
        WorthQueryProductWorldClock::start(),
        super::super::WorthQueryInvalidationResources::install(
            super::super::WorthQueryInvalidationResourceInstallation {
                maximum_marking_work: 1_000_000,
                maximum_preparation_bytes: 64 * 1_024 * 1_024,
                maximum_retained_bytes: 128 * 1_024 * 1_024,
                maximum_retained_positions: 128,
            },
        )
        .expect("the test invalidation resources are valid"),
    )
}
