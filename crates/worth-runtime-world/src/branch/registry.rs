mod close_order;
mod installation;
#[cfg(test)]
pub(crate) mod installation_unwind;
mod installed_entries;
mod reservation;
pub(crate) use installation::ProductBranchInstallationWitness;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation, RuntimeWorldOwnerIdentity};

use super::{
    ProductBranchName, ProductBranchObservation, ProductBranchReferenceCell,
    ProductBranchReferenceRetirement, ProductBranchReferenceSnapshot,
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

pub(crate) enum ProductBranchCurrentnessLookupStop {
    AdmissionRefused,
    AccountingOverflow,
}

#[derive(Debug)]
struct ProductBranchRegistryState {
    owner: RuntimeWorldOwnerIdentity,
    maximum_branches: usize,
    reserved_branches: usize,
    reserved_names: HashSet<String>,
    /// Keyed by the owner-plus-normalized-name identity, so the installed name
    /// index and the branch index are one map rather than two authorities.
    entries: installed_entries::InstalledProductBranches,
    /// Secondary occurrence index. The branch entry remains the sole head
    /// authority; this index only resolves a copyable owner-issued occurrence
    /// token to that entry.
    lifecycles: HashMap<ProductBranchIncarnation, ProductBranchIdentity>,
    root: Option<ProductBranchIdentity>,
}

#[derive(Debug)]
struct ProductBranchRegistryEntry {
    lifecycle: ProductBranchIncarnation,
    cell: ProductBranchReferenceCell,
    /// The exact source commit selected when this occurrence was installed.
    /// Retirement reclaims only descendants of this boundary. Root has no
    /// branch-local source boundary and is left to owner close.
    retirement_boundary: Option<crate::identity::CompositeCommitIdentity>,
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
                entries: installed_entries::InstalledProductBranches::default(),
                lifecycles: HashMap::new(),
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

    pub(crate) fn branch_cell_by_lifecycle(
        &self,
        lifecycle: ProductBranchIncarnation,
    ) -> Option<ProductBranchReferenceCell> {
        self.branch_cell_by_lifecycle_core(lifecycle, None)
            .ok()
            .flatten()
    }

    /// Resolve the same issued occurrence as `branch_cell_by_lifecycle`, with
    /// physical lookup work admitted before either map probes or clones a cell.
    /// A refusal leaves both indexes and the product reference untouched.
    pub(crate) fn branch_cell_by_lifecycle_admitted(
        &self,
        lifecycle: ProductBranchIncarnation,
        prepare: &mut dyn FnMut(u64) -> bool,
    ) -> Result<Option<ProductBranchReferenceCell>, ProductBranchCurrentnessLookupStop> {
        self.branch_cell_by_lifecycle_core(lifecycle, Some(prepare))
    }

    fn branch_cell_by_lifecycle_core(
        &self,
        lifecycle: ProductBranchIncarnation,
        mut prepare: Option<&mut dyn FnMut(u64) -> bool>,
    ) -> Result<Option<ProductBranchReferenceCell>, ProductBranchCurrentnessLookupStop> {
        if let Some(prepare) = prepare.as_mut() {
            if !prepare(1) {
                return Err(ProductBranchCurrentnessLookupStop::AdmissionRefused);
            }
        }
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        // HashMap::capacity is a live-entry capacity, not a bucket count. Two
        // bucket visits per capacity plus one group cover the metadata probe;
        // the fixed occurrence key has no variable payload comparison.
        if let Some(prepare) = prepare.as_mut() {
            let lifecycle_probe = hash_probe_work(state.lifecycles.capacity())
                .and_then(|work| work.checked_add(2))
                .ok_or(ProductBranchCurrentnessLookupStop::AccountingOverflow)?;
            if !prepare(lifecycle_probe) {
                return Err(ProductBranchCurrentnessLookupStop::AdmissionRefused);
            }
        }
        let Some(branch) = state.lifecycles.get(&lifecycle) else {
            return Ok(None);
        };
        if let Some(prepare) = prepare.as_mut() {
            let name_bytes = u64::try_from(branch.name().as_str().len())
                .map_err(|_| ProductBranchCurrentnessLookupStop::AccountingOverflow)?;
            let entry_count = u64::try_from(state.entries.len())
                .map_err(|_| ProductBranchCurrentnessLookupStop::AccountingOverflow)?;
            let entry_probe = hash_probe_work(state.entries.capacity())
                .and_then(|work| work.checked_add(name_bytes))
                .and_then(|work| {
                    work.checked_add(entry_count.checked_mul(name_bytes.checked_add(2)?)?)
                })
                .and_then(|work| work.checked_add(1))
                .ok_or(ProductBranchCurrentnessLookupStop::AccountingOverflow)?;
            if !prepare(entry_probe) {
                return Err(ProductBranchCurrentnessLookupStop::AdmissionRefused);
            }
        }
        Ok(state.entries.get(branch).map(|entry| entry.cell.clone()))
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
    ) -> Result<
        (
            ProductBranchReferenceCell,
            ProductBranchIncarnation,
            ProductBranchReferenceRetirement,
            Option<crate::identity::CompositeCommitIdentity>,
        ),
        ProductBranchRegistryDenial,
    > {
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
        let retirement = state
            .entries
            .get(branch)
            .expect("the observed incarnation was checked under the registry guard")
            .cell
            .retire();
        let entry = release_installed_entry(&mut state, branch)
            .expect("the observed incarnation was checked under the registry guard");
        Ok((
            entry.cell,
            entry.lifecycle,
            retirement,
            entry.retirement_boundary,
        ))
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
        let branches = close_order::non_root_branches(state.entries.keys(), state.root.as_ref());
        let released: Vec<_> = branches
            .iter()
            .map(|branch| {
                release_installed_entry(&mut state, branch)
                    .expect("a branch just read out of the index is still installed")
            })
            .collect();
        drop(state);
        for entry in released {
            drop(entry);
        }
        branches.len()
    }
}

fn hash_probe_work(capacity: usize) -> Option<u64> {
    u64::try_from(capacity)
        .ok()?
        .checked_mul(2)?
        .checked_add(16)
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
