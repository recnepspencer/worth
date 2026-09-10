/// Query's private admission of one exact product branch occurrence.
///
/// The retained World observation owns the component and history obligations;
/// Query can project component bases but cannot mint or reconstruct them.
pub struct WorthQueryProductBranchLease {
    publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
    read: WorthQueryProductObservationLease,
    bridge_source: std::sync::Arc<
        worth_relational::facade::bridge::RelationalBridgeObservationLease,
    >,
}

/// Exact World occurrence retained for read-only Query execution. It carries
/// no Bridge mutation source and therefore cannot enter a publication lane.
pub struct WorthQueryProductObservationLease {
    observation: worth_runtime_world::facade::ProductBranchObservation,
}

impl WorthQueryProductObservationLease {
    pub(crate) fn new(observation: worth_runtime_world::facade::ProductBranchObservation) -> Self {
        Self { observation }
    }

    pub(crate) fn retained_clone(&self) -> Self {
        Self::new(self.observation.clone())
    }

    pub(crate) const fn observation(
        &self,
    ) -> &worth_runtime_world::facade::ProductBranchObservation {
        &self.observation
    }

    pub(crate) fn relational_basis(
        &self,
    ) -> &worth_relational::facade::branch::AdmittedRelationalBranchBasis {
        self.observation.basis().relational_basis()
    }

    pub fn branch_identity(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        self.observation.branch_identity()
    }

    pub fn selected_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.observation.selected_commit()
    }

    pub fn relational_basis_descriptor(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchBasisDescriptor {
        self.relational_basis().descriptor()
    }
}

impl WorthQueryProductBranchLease {
    pub(crate) fn read_lease(&self) -> WorthQueryProductObservationLease {
        self.read.retained_clone()
    }

    pub(crate) fn into_read_lease(self) -> WorthQueryProductObservationLease {
        self.read
    }

    pub(crate) const fn read_lease_ref(&self) -> &WorthQueryProductObservationLease {
        &self.read
    }
    pub(crate) fn retained_clone(&self) -> Self {
        Self {
            publication: self.publication.clone(),
            read: self.read.retained_clone(),
            bridge_source: std::sync::Arc::clone(&self.bridge_source),
        }
    }

    #[doc(hidden)]
    pub fn has_same_selected_occurrence(&self, other: &Self) -> bool {
        self.observation() == other.observation()
    }

    pub(crate) fn new(
        publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        bridge_source: worth_relational::facade::bridge::RelationalBridgeObservationLease,
    ) -> Self {
        let read = WorthQueryProductObservationLease::new(publication.observation().clone());
        Self {
            publication,
            read,
            bridge_source: std::sync::Arc::new(bridge_source),
        }
    }

    pub(crate) fn observation(&self) -> &worth_runtime_world::facade::ProductBranchObservation {
        self.read.observation()
    }

    pub(crate) fn relational_basis(
        &self,
    ) -> &worth_relational::facade::branch::AdmittedRelationalBranchBasis {
        self.observation().basis().relational_basis()
    }

    pub fn branch_identity(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        self.observation().branch_identity()
    }

    pub fn selected_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.observation().selected_commit()
    }

    pub fn relational_basis_descriptor(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchBasisDescriptor {
        self.relational_basis().descriptor()
    }

    #[doc(hidden)]
    pub fn bridge_snapshot_identity(&self) -> &worth_runtime_bridge::facade::TruthSnapshotIdentity {
        self.bridge_source.snapshot_identity()
    }

    pub(crate) fn bridge_source(
        &self,
    ) -> std::sync::Arc<worth_relational::facade::bridge::RelationalBridgeObservationLease> {
        std::sync::Arc::clone(&self.bridge_source)
    }

    pub(crate) fn bridge_source_observation(
        &self,
    ) -> &worth_relational::facade::bridge::RelationalBridgeObservationLease {
        &self.bridge_source
    }

    pub(crate) fn signal_basis(&self) -> &worth_signal::facade::branch::AdmittedSignalBranchBasis {
        self.observation().basis().signal_basis()
    }

    pub(crate) fn publication_binding(
        &self,
    ) -> crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding{
        self.publication.clone()
    }
}
