//! Integration courts grouped by execution cost.

pub(crate) use worth_relational::facade::mvcc::RelationalTransactionIntent;
pub(crate) use {
    worth_runtime_world::facade::CompositeComponentIntent,
    worth_runtime_world::facade::CompositePublicationIntent,
    worth_runtime_world::facade::ProductBranchCreationIntent,
    worth_runtime_world::facade::ProductBranchCreationPlans,
    worth_runtime_world::facade::ProductBranchName,
    worth_runtime_world::facade::RelationalBranchCreationPlan,
    worth_runtime_world::facade::RuntimeWorldBranchBudgetInstallation,
    worth_runtime_world::facade::RuntimeWorldBudgetDenial,
    worth_runtime_world::facade::RuntimeWorldBudgetInstallation,
    worth_runtime_world::facade::RuntimeWorldBudgets,
    worth_runtime_world::facade::RuntimeWorldCancellationSource,
    worth_runtime_world::facade::RuntimeWorldClock,
    worth_runtime_world::facade::RuntimeWorldClockSource,
    worth_runtime_world::facade::RuntimeWorldCustodyBudgetInstallation,
    worth_runtime_world::facade::RuntimeWorldHistoryBudgetInstallation,
    worth_runtime_world::facade::RuntimeWorldInstant,
    worth_runtime_world::facade::RuntimeWorldObservationBudgetInstallation,
    worth_runtime_world::facade::RuntimeWorldPublicationBudgetInstallation,
    worth_runtime_world::facade::RuntimeWorldRecoveryBudgetInstallation,
    worth_runtime_world::facade::RuntimeWorldRetentionBudgetInstallation,
    worth_runtime_world::facade::SignalBranchCreationPlan,
};

pub(crate) struct FixedClock;

impl RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> RuntimeWorldInstant {
        worth_runtime_world::facade::RuntimeWorldInstant::from_ticks(42)
    }
}
