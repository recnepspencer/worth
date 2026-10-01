use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    BTreeNodeCellV1, BTreeNodeV1, DerivedFamilyRootDirectoryV1, DerivedFamilyRootEntry,
    PersistedRecordIdentity,
};

use crate::physical_runtime::{
    layout::{PhysicalIndexPointKey, PhysicalLayoutPagePort},
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, PhysicalRecordId, RecordByteLimit,
    RecordReadLimits, ServingPhysicalRuntime,
};

use super::{
    admit_directory_retirement,
    append::{append_layout_record, LayoutAppendKind},
    publish_derived_directory, retire_selected_tree, AdmittedDirectoryRetirement,
    PhysicalLayoutMaintenanceFailure,
};

const FAMILY_CODE: u16 = 1;

impl ServingPhysicalRuntime {
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_selected_layout_record(&self, record: PersistedRecordIdentity) -> bool {
        let Ok(reader) = self.records() else {
            return false;
        };
        let Some(limit) = RecordByteLimit::new(self.maximum_inline_record_bytes()) else {
            return false;
        };
        reader
            .open(
                PhysicalRecordId::from_persisted(record),
                RecordReadLimits::new(limit),
            )
            .is_ok()
    }

    /// Certification-only fixed topology. All nodes and the directory are
    /// actual classified C.5 appends; normal callers cannot name a root.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_publish_blob_catalog_probe_tree(
        &self,
        height: u8,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
    ) -> Result<Vec<PhysicalIndexPointKey>, PhysicalLayoutMaintenanceFailure> {
        if height != 2 && height != 3 && height != 5 {
            return Err(PhysicalLayoutMaintenanceFailure::TreeHeightLimit);
        }
        let selected = self
            .records()
            .map_err(PhysicalLayoutMaintenanceFailure::RootProtection)?;
        if selected.selected_latest_blob_publication().is_some()
            || selected.selected_derived_family_directory().is_some()
        {
            return Err(PhysicalLayoutMaintenanceFailure::PriorWatermarkMismatch);
        }
        drop(selected);
        let seed = append_node(
            self,
            BTreeNodeV1::leaf(
                FAMILY_CODE,
                vec![BTreeNodeCellV1::leaf(
                    key(250).to_vec(),
                    record_bytes(PersistedRecordIdentity::new([0xE1; 16], 1).unwrap()),
                )],
                None,
                None,
            )
            .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?,
            placement,
            deadline,
        )?;
        let count = 1usize << (height - 1);
        let mut leaves = Vec::with_capacity(count);
        for index in 1..=count {
            let node = BTreeNodeV1::leaf(
                FAMILY_CODE,
                vec![BTreeNodeCellV1::leaf(
                    key(index as u8).to_vec(),
                    record_bytes(seed),
                )],
                None,
                None,
            )
            .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
            leaves.push((index as u8, append_node(self, node, placement, deadline)?));
        }
        let mut level = 1;
        while leaves.len() > 1 {
            let mut parents = Vec::with_capacity(leaves.len() / 2);
            for pair in leaves.chunks_exact(2) {
                let parent = append_node(
                    self,
                    interior(level, pair[0].1, key(pair[1].0), pair[1].1)?,
                    placement,
                    deadline,
                )?;
                parents.push((pair[0].0, parent));
            }
            leaves = parents;
            level += 1;
        }
        let root = leaves[0].1;
        let directory = DerivedFamilyRootDirectoryV1::new(vec![DerivedFamilyRootEntry::new(
            DurableArtifactFamilyId::BlobCatalog,
            root,
        )
        .expect("registered BlobCatalog has a directory tag")])
        .map_err(PhysicalLayoutMaintenanceFailure::DirectoryFormat)?;
        append_layout_record(
            self,
            LayoutAppendKind::DerivedDirectory {
                previous: None,
                replaced_nodes: AdmittedDirectoryRetirement::certification_empty(),
            },
            directory.encode(),
            placement,
            deadline,
        )
        .map_err(PhysicalLayoutMaintenanceFailure::Append)?;
        (1..=count)
            .map(|index| {
                PhysicalIndexPointKey::selected_blob_catalog(
                    self.store_identity(),
                    [index as u8; 16],
                    1,
                )
                .map_err(PhysicalLayoutMaintenanceFailure::PointKey)
            })
            .collect()
    }

    /// Logically drops only derived roots in the selected directory. The
    /// authoritative publication and blob tree remain selected; rebuild must
    /// recover the family roots without consulting old derived leaves.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_drop_blob_derived_roots(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
    ) -> Result<Vec<PersistedRecordIdentity>, PhysicalLayoutMaintenanceFailure> {
        let selected = self
            .records()
            .map_err(PhysicalLayoutMaintenanceFailure::RootProtection)?;
        let source = selected
            .selected_latest_blob_publication()
            .ok_or(PhysicalLayoutMaintenanceFailure::SourceNotSelected)?;
        let previous = selected
            .selected_derived_family_directory()
            .ok_or(PhysicalLayoutMaintenanceFailure::PriorWatermarkMismatch)?;
        let port = PhysicalLayoutPagePort::from_protected_reader(
            self,
            selected,
            self.maximum_inline_record_bytes(),
        )
        .map_err(PhysicalLayoutMaintenanceFailure::DirectoryRead)?;
        let prior_bytes = port
            .read_node(previous.directory_record())
            .map_err(PhysicalLayoutMaintenanceFailure::DirectoryRead)?;
        let prior = DerivedFamilyRootDirectoryV1::decode(prior_bytes.bytes())
            .map_err(PhysicalLayoutMaintenanceFailure::DirectoryFormat)?;
        let retired_selected = prior
            .entries()
            .iter()
            .map(|entry| {
                retire_selected_tree(
                    self,
                    &port,
                    entry.family(),
                    entry.root_record(),
                    placement,
                    deadline,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut pending = prior
            .entries()
            .iter()
            .map(|entry| entry.root_record())
            .collect::<Vec<_>>();
        let mut replaced = Vec::new();
        while let Some(record) = pending.pop() {
            if replaced.len() >= 64 || replaced.contains(&record) {
                return Err(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged);
            }
            let node_bytes = port
                .read_node(record)
                .map_err(PhysicalLayoutMaintenanceFailure::NodeRead)?;
            let node = BTreeNodeV1::decode(node_bytes.bytes())
                .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
            if let Some(first) = node.first_child() {
                pending.push(first);
            }
            for cell in node.cells() {
                if let Some(child) = cell.child() {
                    pending.push(child);
                }
            }
            replaced.push(record);
        }
        let directory = DerivedFamilyRootDirectoryV1::new(Vec::new())
            .map_err(PhysicalLayoutMaintenanceFailure::DirectoryFormat)?
            .with_indexed_through(source);
        let retirement = admit_directory_retirement(
            self,
            &port,
            placement,
            deadline,
            Some(&prior),
            &directory,
            Vec::new(),
            retired_selected,
            Vec::new(),
        )?;
        publish_derived_directory(
            self,
            directory,
            Some(previous),
            retirement,
            placement,
            deadline,
        )
        .map_err(PhysicalLayoutMaintenanceFailure::Append)?;
        replaced.push(previous.directory_record());
        Ok(replaced)
    }
}

fn append_node(
    runtime: &ServingPhysicalRuntime,
    node: BTreeNodeV1,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<PersistedRecordIdentity, PhysicalLayoutMaintenanceFailure> {
    let encoded = node
        .encode(runtime.maximum_layout_node_bytes(placement))
        .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)?;
    append_layout_record(
        runtime,
        LayoutAppendKind::BTreeNode(DurableArtifactFamilyId::BlobCatalog),
        encoded,
        placement,
        deadline,
    )
    .map_err(PhysicalLayoutMaintenanceFailure::Append)
}

fn interior(
    level: u8,
    first: PersistedRecordIdentity,
    separator: [u8; 24],
    right: PersistedRecordIdentity,
) -> Result<BTreeNodeV1, PhysicalLayoutMaintenanceFailure> {
    BTreeNodeV1::interior(
        FAMILY_CODE,
        level,
        first,
        vec![BTreeNodeCellV1::interior(separator.to_vec(), right)],
        None,
        None,
    )
    .map_err(PhysicalLayoutMaintenanceFailure::NodeFormat)
}

fn key(index: u8) -> [u8; 24] {
    let mut key = [0; 24];
    key[..16].fill(index);
    key[16..].copy_from_slice(&1_u64.to_be_bytes());
    key
}

fn record_bytes(record: PersistedRecordIdentity) -> Vec<u8> {
    let mut bytes = vec![0; 24];
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    bytes
}
