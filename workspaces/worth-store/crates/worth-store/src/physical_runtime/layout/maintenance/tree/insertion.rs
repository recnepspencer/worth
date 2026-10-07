use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{BTreeNodeCellV1, BTreeNodeV1, PersistedRecordIdentity};

use crate::physical_runtime::{
    layout::PhysicalLayoutPagePort, AdmittedRecordPlacementPolicy, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};

use super::{
    replacements::ChargedReplacementRecords, PhysicalLayoutMaintenanceFailure, TreeWriter,
    MAXIMUM_HEIGHT,
};

pub(in crate::physical_runtime) struct InsertedLayoutTree<'runtime> {
    pub(super) family: DurableArtifactFamilyId,
    pub(super) source_root: Option<PersistedRecordIdentity>,
    pub(super) root: PersistedRecordIdentity,
    pub(super) replaced: ChargedReplacementRecords<'runtime>,
}

pub(in crate::physical_runtime) enum InsertionSource<'runtime> {
    Root(Option<PersistedRecordIdentity>),
    Continue(InsertedLayoutTree<'runtime>),
}

impl InsertedLayoutTree<'_> {
    pub(in crate::physical_runtime) const fn root(&self) -> PersistedRecordIdentity {
        self.root
    }
}

pub(in crate::physical_runtime) fn insert_registered_node<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    port: &PhysicalLayoutPagePort<'_>,
    family: DurableArtifactFamilyId,
    source: InsertionSource<'runtime>,
    key: Vec<u8>,
    value: Vec<u8>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<InsertedLayoutTree<'runtime>, PhysicalLayoutMaintenanceFailure> {
    let cell = LayoutCellWrite {
        key,
        value,
        superseded: None,
    };
    write_registered_cell(runtime, port, family, source, cell, placement, deadline)
}

/// One leaf cell to write. `superseded` names the exact stale value the cell
/// may replace under its key; any other existing value is a conflict.
pub(in crate::physical_runtime) struct LayoutCellWrite {
    pub(in crate::physical_runtime) key: Vec<u8>,
    pub(in crate::physical_runtime) value: Vec<u8>,
    pub(in crate::physical_runtime) superseded: Option<Vec<u8>>,
}

pub(in crate::physical_runtime) fn write_registered_cell<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    port: &PhysicalLayoutPagePort<'_>,
    family: DurableArtifactFamilyId,
    source: InsertionSource<'runtime>,
    cell: LayoutCellWrite,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<InsertedLayoutTree<'runtime>, PhysicalLayoutMaintenanceFailure> {
    let LayoutCellWrite {
        key,
        value,
        superseded,
    } = cell;
    let registered = runtime
        .registered_btree_family(family)
        .map_err(|_| PhysicalLayoutMaintenanceFailure::UnregisteredFamily(family))?;
    if (key.len(), value.len()) != registered.cell_shape() {
        return Err(PhysicalLayoutMaintenanceFailure::InvalidFamilyCell);
    }
    let (source_root, root, mut replaced) = match source {
        InsertionSource::Root(root) => (root, root, ChargedReplacementRecords::admit(runtime)?),
        InsertionSource::Continue(prior) => {
            if prior.family != family || !prior.replaced.belongs_to(runtime) {
                return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
            }
            (prior.source_root, Some(prior.root), prior.replaced)
        }
    };
    // Continuation capacity belongs to this insertion: deny before even
    // unreachable COW appends, and retain its grant after writer scratch drops.
    replaced.prepare_insertion()?;
    let mut writer = TreeWriter::new(runtime, port, family, placement, deadline, 10)?;
    writer.superseded = superseded;
    let new_root = match root {
        None => {
            let node = BTreeNodeV1::leaf(
                writer.family_code,
                vec![BTreeNodeCellV1::leaf(key, value)],
                None,
                None,
            )
            .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
            writer.append_node(&node)?
        }
        Some(root) => {
            let result = writer.insert_at(root, &key, &value, None, 0)?;
            let new_root = if let Some((separator, right)) = result.split {
                if result.level + 1 >= MAXIMUM_HEIGHT {
                    return Err(PhysicalLayoutMaintenanceFailure::TreeHeightLimit);
                }
                let node = BTreeNodeV1::interior(
                    writer.family_code,
                    result.level + 1,
                    result.root,
                    vec![BTreeNodeCellV1::interior(separator, right)],
                    None,
                    None,
                )
                .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
                writer.append_node(&node)?
            } else {
                result.root
            };
            replaced.append_prepared(result.replaced);
            new_root
        }
    };
    Ok(InsertedLayoutTree {
        family,
        source_root,
        root: new_root,
        replaced,
    })
}
