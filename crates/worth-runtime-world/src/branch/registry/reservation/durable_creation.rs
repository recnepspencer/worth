//! Actual selection installs only the original prepared destination, without cancellation.
use super::*;
impl ProductBranchRegistryReservation {
    pub(crate) fn install_durably_created(
        &mut self,
        source: &ProductBranchObservation,
        branch: ProductBranchIdentity,
        lifecycle: ProductBranchIncarnation,
        cell: &mut Option<ProductBranchReferenceCell>,
        permit: &mut crate::branch::WorldProductCreationPermit,
    ) -> crate::branch::WorldCreationCellRetirement {
        let snapshot = cell
            .as_ref()
            .expect("preselected destination")
            .atomic_snapshot();
        let mut installed = BranchInstallation {
            name: self.name.clone().expect("original reservation"),
            branch,
            lifecycle,
            cell,
            snapshot: snapshot.clone(),
            retirement_boundary: Some(source.selected_commit().clone()),
        };
        self.admits_installation(&installed, false)
            .expect("prepared destination matches its original reservation");
        let registry = self.registry.clone();
        let mut state = registry.state.lock().unwrap_or_else(|e| e.into_inner());
        let (_, retirement) = permit.with_selected_destination(&snapshot, || {
            self.insert_installed(&mut state, &mut installed)
                .expect("exact registry storage prepaid before effects")
        });
        retirement
    }
}
