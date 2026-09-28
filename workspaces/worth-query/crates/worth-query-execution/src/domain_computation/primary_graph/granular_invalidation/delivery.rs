use super::{
    WorthQueryGranularInvalidationInstallation, WorthQueryGranularInvalidationObservation,
};

/// Execution-owned identity of the settled primary truth basis that a
/// transported invalidation batch may be read against.
#[doc(hidden)]
#[derive(Clone)]
pub struct WorthQueryGranularSourceReadBasis {
    snapshot: worth_runtime_bridge::facade::TruthSnapshotIdentity,
    branch: worth_runtime_bridge::facade::TruthBranchIdentity,
    observation: std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease>,
}

impl PartialEq for WorthQueryGranularSourceReadBasis {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot == other.snapshot && self.branch == other.branch
    }
}

impl Eq for WorthQueryGranularSourceReadBasis {}

impl WorthQueryGranularSourceReadBasis {
    pub(in crate::domain_computation::primary_graph) fn new(
        snapshot: worth_runtime_bridge::facade::TruthSnapshotIdentity,
        branch: worth_runtime_bridge::facade::TruthBranchIdentity,
        observation: std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease>,
    ) -> Self {
        Self {
            snapshot,
            branch,
            observation,
        }
    }

    pub fn snapshot(&self) -> &worth_runtime_bridge::facade::TruthSnapshotIdentity {
        &self.snapshot
    }

    pub fn branch(&self) -> &worth_runtime_bridge::facade::TruthBranchIdentity {
        &self.branch
    }

    #[doc(hidden)]
    pub fn retain_observation(
        &self,
    ) -> std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease> {
        std::sync::Arc::clone(&self.observation)
    }
}

/// Opaque execution-owned carrier for one clock observation's granular
/// lower-runtime deliveries.
///
/// The carrier is not Query admission. Query must consume it through its
/// installed invalidation manifest and current authority checks.
pub struct WorthQueryGranularInvalidationDeliveryBatch {
    installation: WorthQueryGranularInvalidationInstallation,
    deliveries: Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>,
    observation: WorthQueryGranularInvalidationObservation,
    source_read_basis: Option<WorthQueryGranularSourceReadBasis>,
}

/// Why two granular invalidation batches could not be merged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorthQueryGranularTransportMergeDenial {
    /// The batches come from different installations.
    ForeignInstallation,
    /// The batches were observed against different source read bases.
    SourceReadBasisMismatch,
}

impl WorthQueryGranularInvalidationDeliveryBatch {
    pub const fn len(&self) -> usize {
        self.deliveries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.deliveries.is_empty()
    }

    pub const fn observation(&self) -> WorthQueryGranularInvalidationObservation {
        self.observation
    }

    pub const fn installation(&self) -> &WorthQueryGranularInvalidationInstallation {
        &self.installation
    }

    #[doc(hidden)]
    pub const fn source_read_basis(&self) -> Option<&WorthQueryGranularSourceReadBasis> {
        self.source_read_basis.as_ref()
    }

    /// Read-only integration inspection for certification and host-owned
    /// transport. Query admission still consumes this batch as a whole; the
    /// borrowed Bridge products cannot authorize Query work or publication.
    #[doc(hidden)]
    pub fn bridge_deliveries(
        &self,
    ) -> &[worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery] {
        &self.deliveries
    }

    #[doc(hidden)]
    pub fn into_bridge_deliveries(
        self,
    ) -> Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery> {
        self.deliveries
    }

    /// Retain a direct-truth transport copy without cloning performed Signal
    /// authority. The exact current installation must already admit this
    /// execution-owned batch.
    #[doc(hidden)]
    pub fn retain_direct_truth_transport(
        &self,
        installation: &WorthQueryGranularInvalidationInstallation,
    ) -> Option<Self> {
        if !installation.admits_batch(self) {
            return None;
        }
        let deliveries = self
            .deliveries
            .iter()
            .map(|delivery| {
                worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery::direct(
                    delivery.correspondence_receipt(),
                )
            })
            .collect::<Vec<_>>();
        Some(Self {
            installation: installation.current(),
            observation: WorthQueryGranularInvalidationObservation::from_deliveries(&deliveries),
            deliveries,
            source_read_basis: self.source_read_basis.clone(),
        })
    }

    /// Merge transport batches only while installation and source basis are
    /// exactly identical. Query still performs semantic convergence and
    /// admission after this transport-only operation.
    #[doc(hidden)]
    pub fn merge_transport_batch(
        mut self,
        other: Self,
    ) -> Result<Self, WorthQueryGranularTransportMergeDenial> {
        if !self
            .installation
            .is_same_current_runtime_as(&other.installation)
        {
            return Err(WorthQueryGranularTransportMergeDenial::ForeignInstallation);
        }
        if self.source_read_basis != other.source_read_basis {
            return Err(WorthQueryGranularTransportMergeDenial::SourceReadBasisMismatch);
        }
        self.deliveries.extend(other.deliveries);
        self.observation =
            WorthQueryGranularInvalidationObservation::from_deliveries(&self.deliveries);
        Ok(self)
    }
}

pub(in crate::domain_computation::primary_graph) fn collect_granular_invalidations(
    installation: WorthQueryGranularInvalidationInstallation,
    deliveries: Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>,
    source_read_basis: Option<WorthQueryGranularSourceReadBasis>,
) -> WorthQueryGranularInvalidationDeliveryBatch {
    let observation = WorthQueryGranularInvalidationObservation::from_deliveries(&deliveries);
    WorthQueryGranularInvalidationDeliveryBatch {
        installation,
        observation,
        deliveries,
        source_read_basis,
    }
}
