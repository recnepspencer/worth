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
        // Cold readmission already authenticated `source` from the exact
        // historical incarnation in the validated journal; a retired parent
        // (or a later same-name incarnation) must not block its surviving child.
        self.insert_installed(&mut state, &mut installed)
    }
}
