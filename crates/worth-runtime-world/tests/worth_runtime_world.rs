#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Integration courts grouped by execution cost.

use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_runtime_world::facade::{
    CompositeComponentIntent, CompositePublicationIntent, ProductBranchCreationIntent,
    ProductBranchCreationPlans, ProductBranchName, RelationalBranchCreationPlan,
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial, RuntimeWorldBudgetInstallation,
    RuntimeWorldBudgets, RuntimeWorldCancellationSource, RuntimeWorldClock,
    RuntimeWorldClockSource, RuntimeWorldCustodyBudgetInstallation,
    RuntimeWorldHistoryBudgetInstallation, RuntimeWorldInstant,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
    SignalBranchCreationPlan,
};

struct FixedClock;

impl RuntimeWorldClockSource for FixedClock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(42)
    }
}
#[path = "worth_runtime_world/fast.rs"]
mod fast;

#[path = "worth_runtime_world/exhaustive.rs"]
mod exhaustive;

#[path = "worth_runtime_world/scale.rs"]
mod scale;

#[path = "worth_runtime_world/compile_fail.rs"]
mod compile_fail;

#[path = "worth_runtime_world/fixtures/runtime_world_certification/test_execution_policy.rs"]
pub(crate) mod test_execution_policy;
