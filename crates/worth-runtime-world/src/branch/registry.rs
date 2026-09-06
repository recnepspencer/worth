mod installation;
#[cfg(test)]
pub(crate) mod installation_unwind;
mod reservation;
pub(crate) use installation::ProductBranchInstallationWitness;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation, RuntimeWorldOwnerIdentity};

use super::{
    ProductBranchName, ProductBranchObservation, ProductBranchReferenceCell,
    ProductBranchReferenceSnapshot,
};

pub(crate) use reservation::{
    ProductBranchRegistryReservation, ProductBranchSourceInstallDenial,
    ProductBranchSourceInstallFailure,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductBranchRegistryDenial {
    ForeignOwner,
    CapacityExhausted,
    AlreadyInstalled,
    ReservationMissing,
    IdentityMismatch,
    NameAlreadyReserved,
    NameAlreadyInstalled,
    BranchAlreadyInstalled,
    LifecycleAlreadyInstalled,
    AlreadyRetired,
}

#[derive(Debug)]
struct ProductBranchRegistryState {
    owner: RuntimeWorldOwnerIdentity,
    maximum_branches: usize,
    reserved_branches: usize,
    reserved_names: HashSet<String>,
    /// Keyed by the owner-plus-normalized-name identity, so the installed name
    /// index and the branch index are one map rather than two authorities.
    entries: HashMap<ProductBranchIdentity, ProductBranchRegistryEntry>,
    lifecycles: HashSet<ProductBranchIncarnation>,
    root: Option<ProductBranchIdentity>,
}

#[derive(Debug)]
struct ProductBranchRegistryEntry {
    lifecycle: ProductBranchIncarnation,
    cell: ProductBranchReferenceCell,
}

/// The only managed owner registry for Runtime World product branches.
///
/// The registry owns only product-reference cells and the name, incarnation,
/// and root indexes over them. It keeps no copy of a head: the cell is the
/// one authority for what a branch carries, and exact reuse resolves the
/// commit from the observation a cell issued. It never owns a component
/// branch and never calls a component owner while its mutex is held.
#[derive(Debug, Clone)]
pub(crate) struct ProductBranchRegistry {
    state: Arc<Mutex<ProductBranchRegistryState>>,
}

impl ProductBranchRegistry {
    pub(crate) fn new(
        owner: RuntimeWorldOwnerIdentity,
        maximum_branches: crate::budget::RuntimeWorldBudgetLimit,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(ProductBranchRegistryState {
                owner,
                maximum_branches: maximum_branches.get(),
                reserved_branches: 0,
                reserved_names: HashSet::new(),
                entries: HashMap::new(),
                lifecycles: HashSet::new(),
                root: None,
            })),
        }
    }

    pub(crate) fn reserve_root(
        &self,
        owner: RuntimeWorldOwnerIdentity,
    ) -> Result<ProductBranchRegistryReservation, ProductBranchRegistryDenial> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if owner != state.owner {
            return Err(ProductBranchRegistryDenial::ForeignOwner);
        }
        if state.root.is_some() {
            return Err(ProductBranchRegistryDenial::AlreadyInstalled);
        }
        reservation::reserve_slot(&mut state)?;
        Ok(ProductBranchRegistryReservation::root(self.clone(), owner))
    }

    pub(crate) fn reserve_branch(
        &self,
        owner: RuntimeWorldOwnerIdentity,
        name: ProductBranchName,
    ) -> Result<ProductBranchRegistryReservation, ProductBranchRegistryDenial> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if owner != state.owner {
            return Err(ProductBranchRegistryDenial::ForeignOwner);
        }
        if state
            .entries
            .contains_key(&ProductBranchIdentity::issued(owner, name.clone()))
        {
            return Err(ProductBranchRegistryDenial::NameAlreadyInstalled);
        }
        if !state.reserved_names.insert(name.as_str().to_owned()) {
            return Err(ProductBranchRegistryDenial::NameAlreadyReserved);
        }
        if let Err(denial) = reservation::reserve_slot(&mut state) {
            state.reserved_names.remove(name.as_str());
            return Err(denial);
        }
        Ok(ProductBranchRegistryReservation::named(
            self.clone(),
            owner,
            name,
        ))
    }

    #[cfg(test)]
    pub(crate) fn root_cell(&self) -> Option<ProductBranchReferenceCell> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state
            .root
            .as_ref()
            .and_then(|branch| state.entries.get(branch))
            .map(|entry| entry.cell.clone())
    }

    pub(crate) fn branch_cell(
        &self,
        branch: &ProductBranchIdentity,
    ) -> Option<ProductBranchReferenceCell> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entries
            .get(branch)
            .map(|entry| entry.cell.clone())
    }

    #[cfg(test)]
    pub(crate) fn branch_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entries
            .len()
    }

    #[cfg(test)]
    pub(crate) fn reserved_branch_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .reserved_branches
    }

    /// Release one installed product reference. The retired incarnation is
    /// returned with the cell because it, not the name-keyed identity, is what
    /// the custody records of this occurrence are keyed by.
    pub(crate) fn retire(
        &self,
        observed: &ProductBranchObservation,
    ) -> Result<(ProductBranchReferenceCell, ProductBranchIncarnation), ProductBranchRegistryDenial>
    {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if observed.owner_identity() != state.owner {
            return Err(ProductBranchRegistryDenial::ForeignOwner);
        }
        let branch = observed.branch_identity();
        // The observation proves this occurrence was installed. Only the live
        // occurrence index is needed to tell whether that same one remains.
        if state
            .entries
            .get(branch)
            .is_none_or(|entry| entry.lifecycle != observed.lifecycle_incarnation())
        {
            return Err(ProductBranchRegistryDenial::AlreadyRetired);
        }
        let entry = release_installed_entry(&mut state, branch)
            .expect("the observed incarnation was checked under the registry guard");
        Ok((entry.cell, entry.lifecycle))
    }

    /// Release every installed non-root product reference and report how many
    /// product-head references that released. Close owns this: the registry is
    /// the only holder of an installed branch's reference cell, so the count is
    /// what close actually let go of, not what a budget says could exist.
    ///
    /// The root is deliberately excluded. It is the world's own reference
    /// rather than a branch a caller created, and the close report describes
    /// created branches.
    pub(crate) fn release_non_root_branches(&self) -> usize {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let root = state.root.clone();
        let branches: Vec<ProductBranchIdentity> = state
            .entries
            .keys()
            .filter(|branch| root.as_ref() != Some(*branch))
            .cloned()
            .collect();
        let released: Vec<_> = branches
            .iter()
            .map(|branch| {
                release_installed_entry(&mut state, branch)
                    .expect("a branch just read out of the index is still installed")
            })
            .collect();
        drop(state);
        drop(released);
        branches.len()
    }
}

/// Take one installed occurrence out of every index that names it. Retirement
/// and close both release a product reference, and both release exactly this
/// much: one authority, so neither can leave an index the other clears.
fn release_installed_entry(
    state: &mut ProductBranchRegistryState,
    branch: &ProductBranchIdentity,
) -> Option<ProductBranchRegistryEntry> {
    let entry = state.entries.remove(branch)?;
    state.lifecycles.remove(&entry.lifecycle);
    if state.root.as_ref() == Some(branch) {
        state.root = None;
    }
    Some(entry)
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
