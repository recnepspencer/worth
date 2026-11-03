use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor,
};
use worth_runtime_bridge::facade::{
    RelationalBridgeObservationLease, RelationalBridgeObservationReleaseReceipt,
    RuntimeBridgeRelationalSource, TruthSnapshotIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryManagedRelationalObservationIdentity {
    descriptor: RelationalBranchBasisDescriptor,
    snapshot: TruthSnapshotIdentity,
}

impl WorthQueryManagedRelationalObservationIdentity {
    pub const fn runtime_instance_id(&self) -> u64 {
        self.descriptor.runtime_instance_id()
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot
    }
}

pub(in crate::domain_computation) struct WorthQueryManagedRelationalObservation {
    identity: WorthQueryManagedRelationalObservationIdentity,
    current_at_admission: bool,
    owner_identity: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    bridge_observation: Option<RelationalBridgeObservationLease>,
}

impl WorthQueryManagedRelationalObservation {
    pub(super) fn retain(
        source: &RuntimeBridgeRelationalSource,
        basis: AdmittedRelationalBranchBasis,
        current_at_admission: bool,
    ) -> Result<Self, worth_relational::facade::branch::RelationalBranchBasisDenial> {
        let descriptor = basis.descriptor().clone();
        let bridge_observation = source.retain_branch_basis_for_bridge(&basis)?;
        Ok(Self {
            identity: WorthQueryManagedRelationalObservationIdentity {
                descriptor,
                snapshot: bridge_observation.snapshot_identity().clone(),
            },
            current_at_admission,
            owner_identity: None,
            bridge_observation: Some(bridge_observation),
        })
    }

    pub(super) fn with_product_owner(
        mut self,
        product: Option<&worth_runtime_world::facade::ProductBranchObservation>,
    ) -> Self {
        self.owner_identity = product.map(|product| product.branch_identity().owner_identity());
        self
    }

    pub(in crate::domain_computation) fn owner_identity(
        &self,
    ) -> Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity> {
        self.owner_identity
    }

    pub fn identity(&self) -> &WorthQueryManagedRelationalObservationIdentity {
        &self.identity
    }

    pub const fn was_current_at_admission(&self) -> bool {
        self.current_at_admission
    }

    pub fn is_live(&self) -> bool {
        self.bridge_observation.is_some()
    }

    pub fn release(mut self) -> WorthQueryManagedRelationalObservationReleaseReceipt {
        let bridge_release = self
            .bridge_observation
            .take()
            .map(RelationalBridgeObservationLease::release);
        WorthQueryManagedRelationalObservationReleaseReceipt { bridge_release }
    }
}

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
