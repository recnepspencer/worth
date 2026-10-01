//! Proof-gated retirement of selected derived paths and directories.

use std::collections::BTreeSet;

mod record_budget;
use record_budget::RetirementRecordBudget;

use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    BTreeNodeKind, DerivedFamilyRootDirectoryV1, PersistedRecordIdentity,
};
use worth_store_physical_integrity::{PhysicalDamageCause, PhysicalIntegrityRejection};

use crate::physical_runtime::{
    layout::PhysicalLayoutPagePort, AdmittedRecordPlacementPolicy, MaintenancePhysicalAllocation,
    MaintenanceRetainedDirectoryCharge, PhysicalMutationDeadline, ServingPhysicalRuntime,
};

use super::{InsertedLayoutTree, TreeWriter, MAXIMUM_HEIGHT};
use crate::physical_runtime::layout::PhysicalLayoutMaintenanceFailure;

pub(in crate::physical_runtime) struct RetiredSelectedTree<'runtime> {
    family: DurableArtifactFamilyId,
    root: PersistedRecordIdentity,
    records: Vec<PersistedRecordIdentity>,
    _allocation: MaintenancePhysicalAllocation<'runtime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferredDerivedRetirementCause {
    NodeDamaged(PhysicalDamageCause),
    InvalidFamilyCell,
    TreeTopologyDamaged,
}

/// Exact old derived root whose closure could not be proved. It remains
/// selected C.5 residue after a replacement D1; no child is added to the
/// classified drop set on this posture.
pub(in crate::physical_runtime) struct DeferredSelectedTreeRetirement {
    family: DurableArtifactFamilyId,
    root: PersistedRecordIdentity,
    cause: DeferredDerivedRetirementCause,
}

pub(in crate::physical_runtime) enum SelectedTreeRetirement<'runtime> {
    Proven(RetiredSelectedTree<'runtime>),
    Deferred(DeferredSelectedTreeRetirement),
}

impl DeferredSelectedTreeRetirement {
    pub(in crate::physical_runtime) const fn family(&self) -> DurableArtifactFamilyId {
        self.family
    }

    pub(in crate::physical_runtime) const fn root(&self) -> PersistedRecordIdentity {
        self.root
    }

    pub(in crate::physical_runtime) const fn cause(&self) -> DeferredDerivedRetirementCause {
        self.cause
    }
}

pub(in crate::physical_runtime) struct AdmittedDirectoryRetirement<'runtime> {
    records: Vec<PersistedRecordIdentity>,
    allocation: Option<MaintenancePhysicalAllocation<'runtime>>,
}

impl AdmittedDirectoryRetirement<'_> {
    pub(in crate::physical_runtime) fn records(&self) -> &[PersistedRecordIdentity] {
        &self.records
    }

    pub(in crate::physical_runtime) fn into_charged_parts(
        self,
    ) -> (
        Vec<PersistedRecordIdentity>,
        Option<MaintenanceRetainedDirectoryCharge>,
    ) {
        (
            self.records,
            self.allocation
                .map(MaintenancePhysicalAllocation::into_retained_directory_charge),
        )
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_empty() -> Self {
        Self {
            records: Vec::new(),
            allocation: None,
        }
    }
}

pub(in crate::physical_runtime) fn retire_selected_tree<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    port: &PhysicalLayoutPagePort<'_>,
    family: DurableArtifactFamilyId,
    root: PersistedRecordIdentity,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<RetiredSelectedTree<'runtime>, PhysicalLayoutMaintenanceFailure> {
    let budget = RetirementRecordBudget::from_port(port);
    let allocation = runtime
        .physical_allocations()
        .admit_maintenance(budget.charge(runtime)?)
        .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
    let writer = TreeWriter::new(runtime, port, family, placement, deadline, 8)?;
    let mut records = BTreeSet::new();
    writer.collect_closure(root, None, 0, &mut records, budget)?;
    budget.before_copy(records.len())?;
    Ok(RetiredSelectedTree {
        family,
        root,
        records: records.into_iter().collect(),
        _allocation: allocation,
    })
}

/// A corrupt derived tree is not a source of blob authority, so rebuild may
/// defer its retirement while replacing its directory root. Resource, C.5
/// acquisition, unknown and unsupported failures do not mint this posture.
pub(in crate::physical_runtime) fn inspect_selected_tree_retirement<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    port: &PhysicalLayoutPagePort<'_>,
    family: DurableArtifactFamilyId,
    root: PersistedRecordIdentity,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<SelectedTreeRetirement<'runtime>, PhysicalLayoutMaintenanceFailure> {
    match retire_selected_tree(runtime, port, family, root, placement, deadline) {
        Ok(proven) => Ok(SelectedTreeRetirement::Proven(proven)),
        Err(failure) => {
            let cause = match failure {
                PhysicalLayoutMaintenanceFailure::NodeIntegrity(
                    PhysicalIntegrityRejection::Damaged(localization),
                ) => DeferredDerivedRetirementCause::NodeDamaged(localization.cause()),
                PhysicalLayoutMaintenanceFailure::InvalidFamilyCell => {
                    DeferredDerivedRetirementCause::InvalidFamilyCell
                }
                PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged => {
                    DeferredDerivedRetirementCause::TreeTopologyDamaged
                }
                other => return Err(other),
            };
            Ok(SelectedTreeRetirement::Deferred(
                DeferredSelectedTreeRetirement {
                    family,
                    root,
                    cause,
                },
            ))
        }
    }
}

pub(in crate::physical_runtime) fn admit_directory_retirement<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    port: &PhysicalLayoutPagePort<'_>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
    old_directory: Option<&DerivedFamilyRootDirectoryV1>,
    new_directory: &DerivedFamilyRootDirectoryV1,
    chains: Vec<InsertedLayoutTree>,
    retired_selected: Vec<RetiredSelectedTree<'runtime>>,
    deferred_selected: Vec<DeferredSelectedTreeRetirement>,
) -> Result<AdmittedDirectoryRetirement<'runtime>, PhysicalLayoutMaintenanceFailure> {
    let budget = RetirementRecordBudget::from_port(port);
    let allocation = runtime
        .physical_allocations()
        .admit_maintenance(budget.charge(runtime)?)
        .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
    let root_for = |directory: Option<&DerivedFamilyRootDirectoryV1>, family| {
        directory.and_then(|directory| {
            directory
                .entries()
                .iter()
                .find(|entry| entry.family() == family)
                .map(|entry| entry.root_record())
        })
    };
    let mut seen = BTreeSet::new();
    let mut dropped = BTreeSet::new();
    let mut retired_families = BTreeSet::new();
    let mut deferred_families = BTreeSet::new();
    for old in &retired_selected {
        if !retired_families.insert(old.family)
            || root_for(old_directory, old.family) != Some(old.root)
        {
            return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
        }
    }
    for old in &deferred_selected {
        if retired_families.contains(&old.family)
            || !deferred_families.insert(old.family)
            || root_for(old_directory, old.family) != Some(old.root)
            || root_for(Some(new_directory), old.family) == Some(old.root)
        {
            return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
        }
    }
    for chain in chains {
        if !seen.insert(chain.family)
            || root_for(Some(new_directory), chain.family) != Some(chain.root)
        {
            return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
        }
        let prior_root = root_for(old_directory, chain.family);
        if prior_root == chain.source_root && retired_families.contains(&chain.family) {
            return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
        }
        if prior_root != chain.source_root {
            let Some(prior_root) = prior_root else {
                return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
            };
            if chain.source_root.is_some() {
                return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
            }
            if let Some(old) = retired_selected
                .iter()
                .find(|old| old.family == chain.family && old.root == prior_root)
            {
                extend_retirement(&mut dropped, &old.records, budget)?;
            } else if !deferred_families.contains(&chain.family) {
                return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
            }
        }
        extend_retirement(&mut dropped, &chain.replaced, budget)?;
    }
    for old in &retired_selected {
        if !seen.contains(&old.family) {
            if root_for(old_directory, old.family) != Some(old.root)
                || root_for(Some(new_directory), old.family).is_some()
            {
                return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
            }
            seen.insert(old.family);
            extend_retirement(&mut dropped, &old.records, budget)?;
        }
    }
    if deferred_families
        .iter()
        .any(|family| !seen.contains(family))
    {
        return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
    }
    for entry in new_directory.entries() {
        if root_for(old_directory, entry.family()) != Some(entry.root_record())
            && !seen.contains(&entry.family())
        {
            return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
        }
    }
    if let Some(old_directory) = old_directory {
        for entry in old_directory.entries() {
            if root_for(Some(new_directory), entry.family()) != Some(entry.root_record())
                && !seen.contains(&entry.family())
            {
                return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
            }
        }
    }
    for entry in new_directory.entries() {
        let closure = retire_selected_tree(
            runtime,
            port,
            entry.family(),
            entry.root_record(),
            placement,
            deadline,
        )?;
        if closure
            .records
            .iter()
            .any(|record| dropped.contains(record))
        {
            return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
        }
    }
    budget.before_copy(dropped.len())?;
    Ok(AdmittedDirectoryRetirement {
        records: dropped.into_iter().collect(),
        allocation: Some(allocation),
    })
}

fn extend_retirement(
    dropped: &mut BTreeSet<PersistedRecordIdentity>,
    records: &[PersistedRecordIdentity],
    budget: RetirementRecordBudget,
) -> Result<(), PhysicalLayoutMaintenanceFailure> {
    for record in records {
        budget.before_unique_insert(dropped.len(), dropped.contains(record))?;
        dropped.insert(*record);
    }
    Ok(())
}

impl TreeWriter<'_, '_> {
    fn collect_closure(
        &self,
        record: PersistedRecordIdentity,
        expected_level: Option<u8>,
        depth: u8,
        visited: &mut BTreeSet<PersistedRecordIdentity>,
        budget: RetirementRecordBudget,
    ) -> Result<(), PhysicalLayoutMaintenanceFailure> {
        if depth >= MAXIMUM_HEIGHT || visited.contains(&record) {
            return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
        }
        budget.before_unique_insert(visited.len(), false)?;
        visited.insert(record);
        let node = self.read_node(record)?;
        if expected_level.is_some_and(|expected| node.level() != expected) {
            return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
        }
        if node.kind() == BTreeNodeKind::Interior {
            let next_level = node
                .level()
                .checked_sub(1)
                .ok_or(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged)?;
            let first = node
                .first_child()
                .ok_or(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged)?;
            self.collect_closure(first, Some(next_level), depth + 1, visited, budget)?;
            for cell in node.cells() {
                self.collect_closure(
                    cell.child()
                        .ok_or(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged)?,
                    Some(next_level),
                    depth + 1,
                    visited,
                    budget,
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "retirement/tests.rs"]
mod tests;
