//! Original managed work and actual live terminal disposition, never a provider codec.
use crate::domain_computation::{
    WorthQueryManagedRunCleanupDisposition, WorthQueryManagedRunCleanupFailureKind,
    WorthQueryManagedRunTerminalKind,
};
use worth_foundational::facade::ManagedRunObligationIdentity;
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
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryManagedRunRecovery {
    Cleanup(WorthQueryManagedRunCleanupFailureKind),
    Yield {
        cause: crate::domain_computation::WorthQueryYieldRecoveryKind,
        state: QueryManagedYieldRecoveryState,
    },
    Readmission(crate::domain_computation::WorthQueryDirectReadmissionRecoveryKind),
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
