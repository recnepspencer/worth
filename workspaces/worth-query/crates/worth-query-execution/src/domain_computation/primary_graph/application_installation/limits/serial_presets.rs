//! Named single-process starting points for application limits.
//!
//! A preset only chooses inputs. Bounded World budgets still install through
//! [`WorthQueryProductWorldResources::install`], the same validation and typed
//! refusal a hand-assembled installation uses.
use std::num::NonZeroUsize;

use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;
use worth_runtime_world::facade::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial, RuntimeWorldBudgetInstallation,
    RuntimeWorldBudgets, RuntimeWorldCustodyBudgetInstallation,
    RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
    RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
    RuntimeWorldRetentionBudgetInstallation,
};
use worth_signal::facade::runtime::SignalConditionalEvaluationBudget;

use super::WorthQueryApplicationLimits;
use crate::domain_computation::execution_runtime::{
    product_world::{WorthQueryProductWorldClock, WorthQueryProductWorldResources},
    WorthQueryApplicationQueryResourceProfile, WorthQueryInvalidationResources,
};

impl WorthQueryApplicationLimits {
    /// The execution policy of both serial presets: every request runs on the
    /// calling thread with canonical bitwise results, accounted without a
    /// policy work or memory ceiling.
    pub const SERIAL_EXECUTION_POLICY: ExecutionRequestPolicy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, None),
    );

    /// One process, serial canonical execution, and no World population
    /// ceilings. Actual storage, accounting, ownership and cancellation still
    /// govern every operation. Queries use the default concurrent graph work
    /// profile; conditional evaluation uses the unlimited development budget.
    pub fn ordinary_serial(clock: WorthQueryAdmissionClock) -> Self {
        Self::new(
            WorthQueryProductWorldResources::new(
                RuntimeWorldBudgets::without_policy_limits(),
                WorthQueryProductWorldClock::start(clock),
                WorthQueryInvalidationResources::new(),
                Self::SERIAL_EXECUTION_POLICY,
            ),
            WorthQueryApplicationQueryResourceProfile::default(),
            SignalConditionalEvaluationBudget::development(),
        )
    }

    /// One process, serial canonical execution, and finite World population
    /// ceilings starting from [`WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS`].
    pub const fn bounded_serial(
        clock: WorthQueryAdmissionClock,
    ) -> WorthQueryBoundedApplicationLimits {
        WorthQueryBoundedApplicationLimits {
            budgets: WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS,
            clock,
        }
    }
}

/// Finite serial limits under construction. Each override replaces exactly
/// the named budget group with the caller's values; nothing is clamped or
/// widened. [`install`](Self::install) validates them all at once.
#[derive(Clone, Debug)]
pub struct WorthQueryBoundedApplicationLimits {
    budgets: RuntimeWorldBudgetInstallation,
    clock: WorthQueryAdmissionClock,
}

impl WorthQueryBoundedApplicationLimits {
    /// The starting budgets of [`WorthQueryApplicationLimits::bounded_serial`].
    pub const DEFAULT_BUDGETS: RuntimeWorldBudgetInstallation = RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 16,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 128,
            history_metadata_bytes: 4 * 1024 * 1024,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 64,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 4,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 4,
            retained_partial_metadata_bytes: 4 * 1024 * 1024,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 64,
            in_flight_pin_acquisition_reservations: 16,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 256,
        },
    };

    /// The budgets [`install`](Self::install) will validate.
    pub const fn budgets(&self) -> &RuntimeWorldBudgetInstallation {
        &self.budgets
    }

    pub const fn with_branches(mut self, branches: RuntimeWorldBranchBudgetInstallation) -> Self {
        self.budgets.branches = branches;
        self
    }

    pub const fn with_history(mut self, history: RuntimeWorldHistoryBudgetInstallation) -> Self {
        self.budgets.history = history;
        self
    }

    pub const fn with_observations(
        mut self,
        observations: RuntimeWorldObservationBudgetInstallation,
    ) -> Self {
        self.budgets.observations = observations;
        self
    }

    pub const fn with_publication(
        mut self,
        publication: RuntimeWorldPublicationBudgetInstallation,
    ) -> Self {
        self.budgets.publication = publication;
        self
    }

    pub const fn with_recovery(mut self, recovery: RuntimeWorldRecoveryBudgetInstallation) -> Self {
        self.budgets.recovery = recovery;
        self
    }

    pub const fn with_retention(
        mut self,
        retention: RuntimeWorldRetentionBudgetInstallation,
    ) -> Self {
        self.budgets.retention = retention;
        self
    }

    pub const fn with_custody(mut self, custody: RuntimeWorldCustodyBudgetInstallation) -> Self {
        self.budgets.custody = custody;
        self
    }

    /// Installs through [`WorthQueryProductWorldResources::install`]: a zero or
    /// oversized budget refuses exactly as it would for explicit resources.
    pub fn install(self) -> Result<WorthQueryApplicationLimits, RuntimeWorldBudgetDenial> {
        WorthQueryProductWorldResources::install(
            self.budgets,
            WorthQueryProductWorldClock::start(self.clock),
            WorthQueryInvalidationResources::new(),
            WorthQueryApplicationLimits::SERIAL_EXECUTION_POLICY,
        )
        .map(|world| {
            WorthQueryApplicationLimits::new(
                world,
                WorthQueryApplicationQueryResourceProfile::default(),
                SignalConditionalEvaluationBudget::development(),
            )
        })
    }
}
