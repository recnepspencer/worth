use std::num::NonZeroU64;

#[cfg(test)]
mod allocation_test_ports;
mod insertion;
mod leaf_cells;
use leaf_cells::{write_leaf_cell, LeafCellWrite};
mod replacements;
pub(in crate::physical_runtime) use insertion::{
    insert_registered_node, write_registered_cell, InsertedLayoutTree, InsertionSource,
    LayoutCellWrite,
};
use replacements::ReplacementPath;
mod retirement;
pub use retirement::DeferredDerivedRetirementCause;
pub(in crate::physical_runtime) use retirement::{
    admit_directory_retirement, inspect_selected_tree_retirement, retire_selected_tree,
    AdmittedDirectoryRetirement, SelectedTreeRetirement,
};

use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    BTreeNodeCellV1, BTreeNodeKind, BTreeNodeV1, PersistedRecordIdentity,
};
use worth_store_physical_integrity::{
    validate_btree_node, BTreeNodeIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
    UntrustedPhysicalArtifact,
};

use crate::physical_runtime::{
    layout::PhysicalLayoutPagePort, AdmittedRecordPlacementPolicy, MaintenancePhysicalAllocation,
    PhysicalMutationDeadline, ServingPhysicalRuntime,
};

use super::{
    append::append_layout_record, append::LayoutAppendKind, PhysicalLayoutMaintenanceFailure,
};

const MAXIMUM_HEIGHT: u8 = 5;
// Covers the 4 GiB / 256 KiB heavy lane even with five COW replacements per
// insertion, while keeping a single directory publication's proof bounded.
const MAXIMUM_RETIREMENT_RECORDS: usize = 131_072;

struct InsertedNode {
    root: PersistedRecordIdentity,
    level: u8,
    split: Option<(Vec<u8>, PersistedRecordIdentity)>,
    changed: bool,
    replaced: ReplacementPath,
}

pub(super) struct TreeWriter<'a, 'runtime> {
    runtime: &'a ServingPhysicalRuntime,
    port: &'a PhysicalLayoutPagePort<'runtime>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
    maximum_bytes: usize,
    family: DurableArtifactFamilyId,
    family_code: u16,
    expected_shape: (usize, usize),
    /// The exact stale leaf value this insertion may replace under its key.
    superseded: Option<Vec<u8>>,
    _allocation: MaintenancePhysicalAllocation<'a>,
}

impl<'a, 'runtime> TreeWriter<'a, 'runtime> {
    pub(super) fn new(
        runtime: &'a ServingPhysicalRuntime,
        port: &'a PhysicalLayoutPagePort<'runtime>,
        family: DurableArtifactFamilyId,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
        charge_pages: u64,
    ) -> Result<Self, PhysicalLayoutMaintenanceFailure> {
        let registered = runtime
            .registered_btree_family(family)
            .map_err(|_| PhysicalLayoutMaintenanceFailure::UnregisteredFamily(family))?;
        let charge =
            NonZeroU64::new(u64::from(runtime.maximum_inline_record_bytes()) * charge_pages)
                .expect("admitted inline page size and charge are nonzero");
        let allocation = runtime
            .physical_allocations()
            .admit_maintenance(charge)
            .map_err(PhysicalLayoutMaintenanceFailure::WriterAllocation)?;
        Ok(Self {
            runtime,
            port,
            placement,
            deadline,
            maximum_bytes: runtime.maximum_layout_node_bytes(placement),
            family,
            family_code: registered.family_code(),
            expected_shape: registered.cell_shape(),
            superseded: None,
            _allocation: allocation,
        })
    }

    fn insert_at(
        &self,
        record: PersistedRecordIdentity,
        key: &[u8],
        value: &[u8],
        expected_level: Option<u8>,
        depth: u8,
    ) -> Result<InsertedNode, PhysicalLayoutMaintenanceFailure> {
        if depth >= MAXIMUM_HEIGHT {
            return Err(PhysicalLayoutMaintenanceFailure::TreeHeightLimit);
        }
        let node = self.read_node(record)?;
        if expected_level.is_some_and(|level| node.level() != level) {
            return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
        }
        match node.kind() {
            BTreeNodeKind::Leaf => self.insert_leaf(record, node, key, value),
            BTreeNodeKind::Interior => self.insert_interior(record, node, key, value, depth),
        }
    }

    pub(super) fn read_node(
        &self,
        record: PersistedRecordIdentity,
    ) -> Result<BTreeNodeV1, PhysicalLayoutMaintenanceFailure> {
        let read = self
            .port
            .read_node(record)
            .map_err(PhysicalLayoutMaintenanceFailure::NodeRead)?;
        let range = PhysicalByteRange::new(0, read.bytes().len() as u64)
            .map_err(|_| PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged)?;
        let scope = PhysicalArtifactScope::btree_node(
            self.port.reader().store_identity(),
            record,
            self.family_code,
            range,
        );
        let (validation, _) = validate_btree_node(
            UntrustedPhysicalArtifact::from_bounded_bytes(read.bytes()),
            scope,
        );
        match validation {
            BTreeNodeIntegrityValidation::Intact(validated) => {
                let node = validated.node();
                if node.cells().iter().any(|cell| {
                    cell.key().len() != self.expected_shape.0
                        || (node.kind() == BTreeNodeKind::Leaf
                            && cell
                                .leaf_value()
                                .is_none_or(|value| value.len() != self.expected_shape.1))
                }) {
                    return Err(PhysicalLayoutMaintenanceFailure::InvalidFamilyCell);
                }
                Ok(node.clone())
            }
            BTreeNodeIntegrityValidation::Rejected(rejection) => {
                Err(PhysicalLayoutMaintenanceFailure::NodeIntegrity(rejection))
            }
        }
    }

    fn insert_leaf(
        &self,
        old_record: PersistedRecordIdentity,
        node: BTreeNodeV1,
        key: &[u8],
        value: &[u8],
    ) -> Result<InsertedNode, PhysicalLayoutMaintenanceFailure> {
        let mut cells = node.cells().to_vec();
        if write_leaf_cell(&mut cells, key, value, self.superseded.as_deref())?
            == LeafCellWrite::Unchanged
        {
            return Ok(InsertedNode {
                root: old_record,
                level: 0,
                split: None,
                changed: false,
                replaced: ReplacementPath::empty(),
            });
        }
        let candidate = BTreeNodeV1::leaf(self.family_code, cells.clone(), None, None)
            .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
        if candidate.encode(self.maximum_bytes).is_ok() {
            return Ok(InsertedNode {
                root: self.append_node(&candidate)?,
                level: 0,
                split: None,
                changed: true,
                replaced: ReplacementPath::one(old_record),
            });
        }
        let right_cells = cells.split_off(cells.len() / 2);
        let separator = right_cells[0].key().to_vec();
        let left = BTreeNodeV1::leaf(self.family_code, cells, None, None)
            .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
        let right = BTreeNodeV1::leaf(self.family_code, right_cells, None, None)
            .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
        let left_record = self.append_node(&left)?;
        let right_record = self.append_node(&right)?;
        Ok(InsertedNode {
            root: left_record,
            level: 0,
            split: Some((separator, right_record)),
            changed: true,
            replaced: ReplacementPath::one(old_record),
        })
    }

    fn insert_interior(
        &self,
        old_record: PersistedRecordIdentity,
        node: BTreeNodeV1,
        key: &[u8],
        value: &[u8],
        depth: u8,
    ) -> Result<InsertedNode, PhysicalLayoutMaintenanceFailure> {
        let level = node.level();
        let mut first_child = node
            .first_child()
            .ok_or(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged)?;
        let mut cells = node.cells().to_vec();
        let position = cells.partition_point(|cell| cell.key() <= key);
        let selected_child = if position == 0 {
            first_child
        } else {
            cells[position - 1]
                .child()
                .ok_or(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged)?
        };
        let inserted = self.insert_at(selected_child, key, value, Some(level - 1), depth + 1)?;
        if !inserted.changed {
            return Ok(InsertedNode {
                root: old_record,
                level,
                split: None,
                changed: false,
                replaced: ReplacementPath::empty(),
            });
        }
        let mut replaced = inserted.replaced;
        replaced.push(old_record);
        if position == 0 {
            first_child = inserted.root;
        } else {
            let separator = cells[position - 1].key().to_vec();
            cells[position - 1] = BTreeNodeCellV1::interior(separator, inserted.root);
        }
        if let Some((separator, right)) = inserted.split {
            cells.insert(position, BTreeNodeCellV1::interior(separator, right));
        }
        let candidate = BTreeNodeV1::interior(
            self.family_code,
            level,
            first_child,
            cells.clone(),
            None,
            None,
        )
        .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
        if candidate.encode(self.maximum_bytes).is_ok() {
            return Ok(InsertedNode {
                root: self.append_node(&candidate)?,
                level,
                split: None,
                changed: true,
                replaced,
            });
        }
        if cells.len() < 3 {
            return Err(PhysicalLayoutMaintenanceFailure::TreeHeightLimit);
        }
        let right_cells = cells.split_off(cells.len() / 2 + 1);
        let promoted = cells.pop().expect("interior split has a middle cell");
        let separator = promoted.key().to_vec();
        let right_first = promoted
            .child()
            .ok_or(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged)?;
        let left = BTreeNodeV1::interior(self.family_code, level, first_child, cells, None, None)
            .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
        let right = BTreeNodeV1::interior(
            self.family_code,
            level,
            right_first,
            right_cells,
            None,
            None,
        )
        .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
        Ok(InsertedNode {
            root: self.append_node(&left)?,
            level,
            split: Some((separator, self.append_node(&right)?)),
            changed: true,
            replaced,
        })
    }

    fn append_node(
        &self,
        node: &BTreeNodeV1,
    ) -> Result<PersistedRecordIdentity, PhysicalLayoutMaintenanceFailure> {
        let encoded = node
            .encode(self.maximum_bytes)
            .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
        append_layout_record(
            self.runtime,
            LayoutAppendKind::BTreeNode(self.family),
            encoded,
            self.placement,
            self.deadline,
        )
        .map_err(PhysicalLayoutMaintenanceFailure::Append)
    }
}
