//! Moved whole from `domain_computation::managed_run::relational_observation`: the items below and every impl they own.
use worth_runtime_bridge::facade::RelationalBridgeObservationReleaseReceipt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryManagedRelationalObservationReleaseReceipt {
    bridge_release: Option<RelationalBridgeObservationReleaseReceipt>,
}

impl WorthQueryManagedRelationalObservationReleaseReceipt {
    pub fn released(&self) -> bool {
        self.bridge_release
            .as_ref()
            .is_some_and(RelationalBridgeObservationReleaseReceipt::released)
    }
}
