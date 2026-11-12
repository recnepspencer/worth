//! Cold selection installs an admitted destination without consulting live currentness.
use super::*;
impl ProductBranchRegistryReservation {
    pub(crate) fn install_readmitted_branch(
        &mut self,
        source: &ProductBranchObservation,
        branch: ProductBranchIdentity,
        lifecycle: ProductBranchIncarnation,
        cell: &mut Option<ProductBranchReferenceCell>,
    ) -> Result<(), ProductBranchRegistryDenial> {
        if source.owner_identity() != self.owner {
            return Err(ProductBranchRegistryDenial::ForeignOwner);
        }
        let mut installed = BranchInstallation {
            name: self
                .name
                .clone()
                .ok_or(ProductBranchRegistryDenial::ReservationMissing)?,
            branch,
            lifecycle,
            snapshot: cell
                .as_ref()
                .expect("admitted original destination")
                .atomic_snapshot(),
            cell,
            retirement_boundary: Some(source.selected_commit().clone()),
        };
        self.admits_installation(&installed, false)?;
        let registry = self.registry.clone();
        let mut state = registry
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let source_cell = state
            .entries
            .get(source.branch_identity())
            .ok_or(ProductBranchRegistryDenial::IdentityMismatch)?;
        if source_cell.lifecycle != source.lifecycle_incarnation() {
            return Err(ProductBranchRegistryDenial::IdentityMismatch);
        }
        self.insert_installed(&mut state, &mut installed)
    }
}
