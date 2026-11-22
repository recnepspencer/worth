//! Original managed work and actual live terminal disposition, never a provider codec.
use worth_foundational::facade::ManagedRunObligationIdentity;
use worth_query_execution_foundation::facade::domain_computation::managed_run_vocabulary::{
    WorthQueryManagedRunCleanupDisposition, WorthQueryManagedRunCleanupFailureKind,
    WorthQueryManagedRunTerminalKind,
};
pub(super) mod codec;
pub(super) mod fold;
mod release;
pub use release::{QueryManagedExecutionRelease, QueryManagedRunResources};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryManagedRunObligation {
    pub identity: ManagedRunObligationIdentity,
    pub logical_run: String,
    pub original_attempt: String,
    pub attempt: String,
    pub posture: QueryManagedRunPosture,
    pub original: Option<super::QueryManagedCleanupOriginal>,
    pub recovery: Option<QueryManagedRunRecovery>,
    /// Recorded once, before service, when the incarnation holding this run
    /// ended. The posture, original and recovery cause stay as recorded.
    pub cold_release: Option<super::QueryColdReleaseTerminal>,
}
impl QueryManagedRunObligation {
    /// The live owner recorded its release, or the run's incarnation ended.
    pub const fn released(&self) -> bool {
        self.posture.resources_released() || self.cold_release.is_some()
    }
    /// R7: running work is abandoned; any other pending posture already
    /// recorded its terminal and only its release was outstanding.
    pub(super) const fn cold_release_terminal(&self) -> super::QueryColdReleaseTerminal {
        match self.posture {
            QueryManagedRunPosture::Active => super::QueryColdReleaseTerminal::Abandoned,
            _ => super::QueryColdReleaseTerminal::ReleaseOutstanding,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryManagedRunRecovery {
    Cleanup(WorthQueryManagedRunCleanupFailureKind),
    Yield {
        cause: worth_query_execution_foundation::facade::domain_computation::managed_run_vocabulary::WorthQueryYieldRecoveryKind,
        state: QueryManagedYieldRecoveryState,
    },
    Readmission(worth_query_execution_foundation::facade::domain_computation::managed_run_vocabulary::WorthQueryDirectReadmissionRecoveryKind),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryManagedYieldRecoveryState {
    Running,
    Terminalized,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryManagedWorkflowCleanupRoute {
    Terminal(WorthQueryManagedRunTerminalKind),
    Yielded,
    Readmission,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryManagedRunPosture {
    Active,
    CleanupRequired {
        terminal: WorthQueryManagedRunTerminalKind,
        failure: WorthQueryManagedRunCleanupFailureKind,
    },
    DirectCleanup {
        terminal: WorthQueryManagedRunTerminalKind,
        disposition: WorthQueryManagedRunCleanupDisposition,
        resources: QueryManagedRunResources,
    },
    YieldedCleanup {
        disposition: WorthQueryManagedRunCleanupDisposition,
        resources: QueryManagedRunResources,
    },
    ReadmissionCleanup {
        disposition: WorthQueryManagedRunCleanupDisposition,
        resources: QueryManagedRunResources,
    },
    WorkflowCleanup {
        route: QueryManagedWorkflowCleanupRoute,
        disposition: WorthQueryManagedRunCleanupDisposition,
        resources: QueryManagedRunResources,
        artifacts_released: bool,
    },
    AdmissionReleased,
}
impl QueryManagedRunPosture {
    pub(super) fn keeps_terminal_route(self, after: Self) -> bool {
        match (self, after) {
            (Self::Active, next) => next != Self::Active,
            (
                Self::CleanupRequired {
                    terminal: before, ..
                },
                Self::CleanupRequired {
                    terminal: after, ..
                }
                | Self::DirectCleanup {
                    terminal: after, ..
                },
            )
            | (
                Self::DirectCleanup {
                    terminal: before, ..
                },
                Self::DirectCleanup {
                    terminal: after, ..
                },
            ) => before == after,
            (Self::YieldedCleanup { .. }, Self::YieldedCleanup { .. })
            | (Self::ReadmissionCleanup { .. }, Self::ReadmissionCleanup { .. }) => true,
            (
                Self::WorkflowCleanup { route: before, .. },
                Self::WorkflowCleanup { route: after, .. },
            ) => before == after,
            _ => false,
        }
    }
    pub const fn resources_released(self) -> bool {
        match self {
            Self::DirectCleanup { resources, .. }
            | Self::YieldedCleanup { resources, .. }
            | Self::ReadmissionCleanup { resources, .. } => resources.released(),
            Self::WorkflowCleanup {
                resources,
                artifacts_released,
                ..
            } => artifacts_released && resources.released(),
            Self::AdmissionReleased => true,
            Self::Active | Self::CleanupRequired { .. } => false,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryManagedRunRecord {
    Retain(QueryManagedRunObligation),
    Handoff {
        identity: ManagedRunObligationIdentity,
        posture: QueryManagedRunPosture,
        attempt: String,
    },
    Original {
        identity: ManagedRunObligationIdentity,
        original: super::QueryManagedCleanupOriginal,
    },
    Recovery {
        identity: ManagedRunObligationIdentity,
        cause: QueryManagedRunRecovery,
    },
    Discharge {
        identity: ManagedRunObligationIdentity,
    },
}
