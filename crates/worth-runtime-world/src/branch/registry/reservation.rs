use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation};
use std::sync::Arc;

use super::{
    ProductBranchInstallationWitness, ProductBranchName, ProductBranchObservation,
    ProductBranchReferenceCell, ProductBranchReferenceSnapshot, ProductBranchRegistry,
    ProductBranchRegistryDenial, ProductBranchRegistryEntry, ProductBranchRegistryState,
};

#[must_use = "a branch reservation must be installed or dropped"]
pub(crate) struct ProductBranchRegistryReservation {
    pub(super) registry: ProductBranchRegistry,
    owner: crate::identity::RuntimeWorldOwnerIdentity,
    name: Option<ProductBranchName>,
    root: bool,
    armed: bool,
    installation: Option<Arc<ProductBranchInstallationWitness>>,
}

impl std::fmt::Debug for ProductBranchRegistryReservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProductBranchRegistryReservation")
            .field("owner", &self.owner)
            .field("name", &self.name)
            .field("root", &self.root)
            .field("armed", &self.armed)
            .finish_non_exhaustive()
    }
}

impl Drop for ProductBranchRegistryReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(name) = &self.name {
            state.reserved_names.remove(name.as_str());
        }
        state.reserved_branches = state
            .reserved_branches
            .checked_sub(1)
            .expect("a live branch reservation owns one slot");
        self.armed = false;
    }
}

#[derive(Debug)]
pub(crate) enum ProductBranchSourceInstallDenial {
    Cancelled,
    Registry(ProductBranchRegistryDenial),
    SourceRetired,
    SourceDisplaced(ProductBranchReferenceSnapshot),
}

/// The failed reservation returns; the borrowed cell remains in caller custody.
#[derive(Debug)]
pub(crate) struct ProductBranchSourceInstallFailure {
    pub(crate) reservation: ProductBranchRegistryReservation,
    pub(crate) denial: ProductBranchSourceInstallDenial,
}

struct BranchInstallation<'a> {
    name: ProductBranchName,
    branch: ProductBranchIdentity,
    lifecycle: ProductBranchIncarnation,
    cell: &'a mut Option<ProductBranchReferenceCell>,
    snapshot: ProductBranchReferenceSnapshot,
    retirement_boundary: Option<crate::identity::CompositeCommitIdentity>,
}

impl ProductBranchRegistryReservation {
    pub(super) fn root(
        registry: ProductBranchRegistry,
        owner: crate::identity::RuntimeWorldOwnerIdentity,
    ) -> Self {
        Self {
            registry,
            owner,
            name: None,
            root: true,
            armed: true,
            installation: None,
        }
    }

    pub(super) fn named(
        registry: ProductBranchRegistry,
        owner: crate::identity::RuntimeWorldOwnerIdentity,
        name: ProductBranchName,
    ) -> Self {
        Self {
            registry,
            owner,
            name: Some(name),
            root: false,
            armed: true,
            installation: None,
        }
    }

    pub(crate) fn bind_creation_destination(
        &mut self,
        branch: ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
    ) -> Result<Arc<ProductBranchInstallationWitness>, ProductBranchRegistryDenial> {
        if self.root
            || !self.armed
            || self.installation.is_some()
            || branch.owner_identity() != self.owner
            || incarnation.owner_identity() != self.owner
            || self.name.as_ref() != Some(branch.name())
        {
            return Err(ProductBranchRegistryDenial::IdentityMismatch);
        }
        let witness = ProductBranchInstallationWitness::reserve(branch, incarnation);
        self.installation = Some(Arc::clone(&witness));
        Ok(witness)
    }

    pub(crate) fn install_root(
        mut self,
        name: ProductBranchName,
        branch: ProductBranchIdentity,
        lifecycle: ProductBranchIncarnation,
        cell: ProductBranchReferenceCell,
    ) -> Result<(), (Self, ProductBranchRegistryDenial)> {
        let snapshot = cell.atomic_snapshot();
        let mut cell = Some(cell);
        let mut installed = BranchInstallation {
            name,
            branch,
            lifecycle,
            cell: &mut cell,
            snapshot,
            retirement_boundary: None,
        };
        if let Err(denial) = self.admits_installation(&installed, true) {
            return Err((self, denial));
        }
        let registry = self.registry.clone();
        let result = self.insert_installed(
            &mut registry.state.lock().unwrap_or_else(|e| e.into_inner()),
            &mut installed,
        );
        result.map_err(|denial| (self, denial))
    }

    /// Keep the destination in the owner's lease through validation and the
    /// source guard. Only actual insertion takes it. Registry then source-cell
    /// is the existing lock order; neither lock spans a component-owner call.
    pub(crate) fn install_from_source(
        mut self,
        source: &ProductBranchObservation,
        branch: ProductBranchIdentity,
        lifecycle: ProductBranchIncarnation,
        cell: &mut Option<ProductBranchReferenceCell>,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<(), ProductBranchSourceInstallFailure> {
        let result =
            self.install_from_source_borrowed(source, branch, lifecycle, cell, cancellation);
        result.map_err(|denial| ProductBranchSourceInstallFailure {
            reservation: self,
            denial,
        })
    }

    fn install_from_source_borrowed(
        &mut self,
        source: &ProductBranchObservation,
        branch: ProductBranchIdentity,
        lifecycle: ProductBranchIncarnation,
        cell: &mut Option<ProductBranchReferenceCell>,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<(), ProductBranchSourceInstallDenial> {
        let name = self
            .name
            .clone()
            .ok_or(ProductBranchSourceInstallDenial::Registry(
                ProductBranchRegistryDenial::ReservationMissing,
            ))?;
        let snapshot = cell
            .as_ref()
            .expect("the caller retains its destination cell")
            .atomic_snapshot();
        let mut installed = BranchInstallation {
            name,
            branch,
            lifecycle,
            cell,
            snapshot,
            retirement_boundary: Some(source.selected_commit().clone()),
        };
        self.admits_installation(&installed, false)
            .map_err(ProductBranchSourceInstallDenial::Registry)?;
        let registry = self.registry.clone();
        let mut state = registry.state.lock().unwrap_or_else(|e| e.into_inner());
        let source_cell = state
            .entries
            .get(source.branch_identity())
            .map(|entry| entry.cell.clone())
            .ok_or(ProductBranchSourceInstallDenial::SourceRetired)?;
        match source_cell.while_current(source, &mut installed, |installed| {
            if cancellation.is_cancelled() {
                return Err(ProductBranchSourceInstallDenial::Cancelled);
            }
            self.insert_installed(&mut state, installed)
                .map_err(ProductBranchSourceInstallDenial::Registry)
        }) {
            Ok(result) => result,
            Err((observed, _)) => Err(ProductBranchSourceInstallDenial::SourceDisplaced(observed)),
        }
    }

    fn admits_installation(
        &self,
        installed: &BranchInstallation<'_>,
        root: bool,
    ) -> Result<(), ProductBranchRegistryDenial> {
        if self.root != root || !self.armed {
            return Err(ProductBranchRegistryDenial::ReservationMissing);
        }
        if installed.branch.owner_identity() != self.owner
            || installed.lifecycle.owner_identity() != self.owner
            || (!root && self.name.as_ref() != Some(&installed.name))
            || installed.branch.name() != &installed.name
            || installed.snapshot.owner_identity() != self.owner
            || installed.snapshot.branch_identity() != &installed.branch
            || installed.snapshot.lifecycle_incarnation() != installed.lifecycle
            || self
                .installation
                .as_ref()
                .is_some_and(|w| !w.admits(&installed.snapshot))
        {
            return Err(ProductBranchRegistryDenial::IdentityMismatch);
        }
        Ok(())
    }

    fn insert_installed(
        &mut self,
        state: &mut ProductBranchRegistryState,
        installed: &mut BranchInstallation<'_>,
    ) -> Result<(), ProductBranchRegistryDenial> {
        if self.root && state.root.is_some() {
            return Err(ProductBranchRegistryDenial::AlreadyInstalled);
        }
        if state.reserved_branches == 0 {
            return Err(ProductBranchRegistryDenial::ReservationMissing);
        }
        if state.entries.contains_key(&installed.branch) {
            return Err(ProductBranchRegistryDenial::BranchAlreadyInstalled);
        }
        if state.lifecycles.contains_key(&installed.lifecycle) {
            return Err(ProductBranchRegistryDenial::LifecycleAlreadyInstalled);
        }
        if self.root && state.reserved_names.contains(installed.name.as_str()) {
            return Err(ProductBranchRegistryDenial::NameAlreadyReserved);
        }
        if !self.root && !state.reserved_names.contains(installed.name.as_str()) {
            return Err(ProductBranchRegistryDenial::ReservationMissing);
        }
        // Both maps' spare storage was reserved before effects. No allocation,
        // callback or destructor separates taking custody and stamping success.
        let cell = installed
            .cell
            .take()
            .expect("validated destination remains borrowed");
        state.entries.insert(
            installed.branch.clone(),
            ProductBranchRegistryEntry {
                lifecycle: installed.lifecycle,
                cell,
                retirement_boundary: installed.retirement_boundary.clone(),
            },
        );
        state
            .lifecycles
            .insert(installed.lifecycle, installed.branch.clone());
        state.reserved_branches -= 1;
        self.armed = false;
        if let Some(witness) = &self.installation {
            witness.record_installation(installed.snapshot.selected_commit().clone());
        }
        state.reserved_names.remove(installed.name.as_str());
        if self.root {
            state.root = Some(installed.branch.clone());
        }
        #[cfg(test)]
        if self.installation.is_some() {
            super::installation_unwind::after_installed();
        }
        Ok(())
    }
}

pub(super) fn reserve_slot(
    state: &mut ProductBranchRegistryState,
) -> Result<(), ProductBranchRegistryDenial> {
    if state.entries.len().saturating_add(state.reserved_branches) >= state.maximum_branches {
        return Err(ProductBranchRegistryDenial::CapacityExhausted);
    }
    let reserved = state
        .reserved_branches
        .checked_add(1)
        .ok_or(ProductBranchRegistryDenial::CapacityExhausted)?;
    state
        .entries
        .try_reserve(reserved)
        .map_err(|_| ProductBranchRegistryDenial::CapacityExhausted)?;
    state
        .lifecycles
        .try_reserve(reserved)
        .map_err(|_| ProductBranchRegistryDenial::CapacityExhausted)?;
    state.reserved_branches = reserved;
    Ok(())
}
