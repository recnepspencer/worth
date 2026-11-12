//! Actual managed cleanup ownership and negative completion evidence.
pub use crate::domain_computation::{
    primary_graph::{
        WorthQueryApplicationCleanupRequired, WorthQueryApplicationManagedCleanup,
        WorthQueryApplicationManagedCleanupReceipt, WorthQueryCleanupDispatchDenial,
        WorthQueryCleanupDispatchPosture, WorthQueryManagedCleanupRegistrationDenial,
        WorthQueryManagedCleanupScheduleForTest,
    },
    WorthQueryManagedRunCleanupFailureKind, WorthQueryMutationCleanupFailure,
    WorthQueryMutationCleanupFailureKind,
};
