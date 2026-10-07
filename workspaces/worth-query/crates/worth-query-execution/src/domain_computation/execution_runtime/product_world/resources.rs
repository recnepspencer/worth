use std::sync::Arc;

use worth_execution::ExecutionAuthority;
use worth_foundational::ExecutionRequestPolicy;
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
    execution: WorthQueryProductExecution,
}

/// How the World's requests run, handed to its builder unchanged: with no
/// policy they run unbounded on the calling thread; with one, on the calling
/// thread within it, or leased from the host's authority when one is
/// installed beside it. An authority without a policy is refused at
/// installation.
#[derive(Clone, Default)]
pub(crate) struct WorthQueryProductExecution {
    pub(crate) authority: Option<Arc<ExecutionAuthority>>,
    pub(crate) policy: Option<ExecutionRequestPolicy>,
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
            execution: WorthQueryProductExecution::default(),
        }
    }

    /// Installs the host's process authority, from which every request
    /// leases the installed policy's budget.
    pub fn with_execution_authority(mut self, authority: Arc<ExecutionAuthority>) -> Self {
        self.execution.authority = Some(authority);
        self
    }

    /// Installs the policy every request runs under.
    pub fn with_execution_policy(mut self, policy: ExecutionRequestPolicy) -> Self {
        self.execution.policy = Some(policy);
        self
    }

    pub fn invalidation_resources(&self) -> super::super::WorthQueryInvalidationResources {
        self.invalidation.clone()
    }

    /// The invalidation window's retained positions and the World's retained
    /// commits, when the window holds more than the World keeps.
    pub(crate) fn invalidation_window_past_history(&self) -> Option<(usize, usize)> {
        let positions = self.invalidation.installation().maximum_retained_positions;
        let commits = self.budgets.retained_composite_commits().get();
        (positions > commits).then_some((positions, commits))
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RuntimeWorldBudgets,
        WorthQueryProductWorldClock,
        super::super::WorthQueryInvalidationResources,
        WorthQueryProductExecution,
    ) {
        (self.budgets, self.clock, self.invalidation, self.execution)
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
