use worth_store_physical_format::{BTreeNodeKind, BTreeNodeV1, PersistedRecordIdentity};
use worth_store_physical_integrity::{
    validate_btree_node, BTreeNodeIntegrityValidation, PhysicalArtifactScope, PhysicalByteRange,
    PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};

use crate::physical_runtime::{
    artifact_family::{RegisteredDerivedFamily, RegisteredIndexShape},
    layout::{PhysicalIndexPointKey, PhysicalLayoutDenial, PhysicalLayoutPagePort},
    RecordReadObservation,
};

// The default 16 KiB DedupeIndex leaf has fanout below 256 and may need a
// fifth level at 2^32 entries. The 64 KiB profile retains the height-4 bound.
const MAX_BTREE_HEIGHT: u64 = 5;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalBTreeReadCounters {
    page_touches: u64,
    protected_bytes_read: u64,
    node_decodes: u64,
    key_comparisons: u64,
}

impl PhysicalBTreeReadCounters {
    pub const fn page_touches(self) -> u64 {
        self.page_touches
    }

    /// Completed protected payload bytes plus executed manifest read bytes.
    /// This is not a device-transfer claim; warm pages may incur no device I/O.
    pub const fn protected_bytes_read(self) -> u64 {
        self.protected_bytes_read
    }

    pub const fn node_decodes(self) -> u64 {
        self.node_decodes
    }

    pub const fn key_comparisons(self) -> u64 {
        self.key_comparisons
    }

    pub(super) fn observe_read(&mut self, observation: RecordReadObservation) {
        self.page_touches += observation.touched_pages();
        self.protected_bytes_read += observation.bytes_completed() + observation.manifest_bytes();
    }

    pub(super) fn observe_comparison(&mut self) {
        self.key_comparisons += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalBTreePointObservation {
    selected_record: Option<PersistedRecordIdentity>,
    counters: PhysicalBTreeReadCounters,
}

impl PhysicalBTreePointObservation {
    pub const fn selected_record(self) -> Option<PersistedRecordIdentity> {
        self.selected_record
    }

    pub const fn counters(self) -> PhysicalBTreeReadCounters {
        self.counters
    }
}

/// One registered B-tree viewed under a single protected selected C.5 root.
pub struct PhysicalBTreeIndex<'access, 'runtime> {
    pub(super) port: &'access PhysicalLayoutPagePort<'runtime>,
    pub(super) registered: RegisteredDerivedFamily,
    pub(super) root: Option<PersistedRecordIdentity>,
    pub(super) damaged_node: std::cell::Cell<Option<PhysicalArtifactScope>>,
    #[cfg(feature = "certification-test-authority")]
    extra_protected_read: std::cell::Cell<bool>,
}

impl<'access, 'runtime> PhysicalBTreeIndex<'access, 'runtime> {
    pub(in crate::physical_runtime) fn from_selected_root(
        port: &'access PhysicalLayoutPagePort<'runtime>,
        registered: RegisteredDerivedFamily,
        root: Option<PersistedRecordIdentity>,
    ) -> Self {
        Self {
            port,
            registered,
            root,
            damaged_node: std::cell::Cell::new(None),
            #[cfg(feature = "certification-test-authority")]
            extra_protected_read: std::cell::Cell::new(false),
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub fn certification_inject_extra_protected_page_read(&self) {
        self.extra_protected_read.set(true);
    }

    pub fn point(
        &self,
        key: PhysicalIndexPointKey,
    ) -> Result<PhysicalBTreePointObservation, PhysicalLayoutDenial> {
        self.damaged_node.set(None);
        self.registered
            .admit_shape(RegisteredIndexShape::Point)
            .map_err(|_| PhysicalLayoutDenial::ShapeNotAdmitted(self.registered.family()))?;
        if key.family() != self.registered.family() {
            return Err(PhysicalLayoutDenial::ShapeNotAdmitted(
                self.registered.family(),
            ));
        }
        if key.store() != self.port.reader().store_identity() {
            return Err(PhysicalLayoutDenial::ForeignStore);
        }
        let (value, counters) = self.point_raw(&key.canonical_bytes())?;
        let selected_record = value.as_deref().map(decode_leaf_record).transpose()?;
        Ok(PhysicalBTreePointObservation {
            selected_record,
            counters,
        })
    }

    /// Store-internal value lookup for registered families whose leaf schema
    /// is richer than the public blob-catalog publication RecordId.
    pub(in crate::physical_runtime) fn point_raw(
        &self,
        key_bytes: &[u8],
    ) -> Result<(Option<Vec<u8>>, PhysicalBTreeReadCounters), PhysicalLayoutDenial> {
        self.damaged_node.set(None);
        self.registered
            .admit_shape(RegisteredIndexShape::Point)
            .map_err(|_| PhysicalLayoutDenial::ShapeNotAdmitted(self.registered.family()))?;
        let expected_key_bytes = self.registered.cell_shape().0;
        if key_bytes.len() != expected_key_bytes {
            return Err(PhysicalLayoutDenial::NodeFamilyMismatch);
        }
        let mut counters = PhysicalBTreeReadCounters::default();
        let Some(mut record) = self.root else {
            return Ok((None, counters));
        };
        let mut expected_level = None;
        loop {
            if counters.node_decodes >= MAX_BTREE_HEIGHT {
                return Err(PhysicalLayoutDenial::TreeDepthExceeded);
            }
            let node = self.read_selected_node(record, &mut counters)?;
            if expected_level.is_some_and(|level| node.level() != level) {
                return Err(PhysicalLayoutDenial::TreeTopologyDamaged);
            }
            match node.kind() {
                BTreeNodeKind::Leaf => {
                    let value = find_leaf_value(node.cells(), key_bytes, &mut counters)?;
                    return Ok((value, counters));
                }
                BTreeNodeKind::Interior => {
                    expected_level = Some(node.level() - 1);
                    record =
                        select_child(node.cells(), node.first_child(), key_bytes, &mut counters)?;
                }
            }
        }
    }

    pub(super) fn read_selected_node(
        &self,
        record: PersistedRecordIdentity,
        counters: &mut PhysicalBTreeReadCounters,
    ) -> Result<BTreeNodeV1, PhysicalLayoutDenial> {
        self.damaged_node.set(None);
        let read = self
            .port
            .read_node(record)
            .map_err(PhysicalLayoutDenial::NodeRead)?;
        counters.observe_read(read.observation());
        #[cfg(feature = "certification-test-authority")]
        if self.extra_protected_read.replace(false) {
            let repeated = self
                .port
                .read_node(record)
                .map_err(PhysicalLayoutDenial::NodeRead)?;
            counters.observe_read(repeated.observation());
        }
        let range = PhysicalByteRange::new(0, read.bytes().len() as u64)
            .map_err(|_| PhysicalLayoutDenial::NodeFamilyMismatch)?;
        let scope = PhysicalArtifactScope::btree_node(
            self.port.reader().store_identity(),
            record,
            self.registered.family_code(),
            range,
        );
        let (validation, _) = validate_btree_node(
            UntrustedPhysicalArtifact::from_bounded_bytes(read.bytes()),
            scope,
        );
        match validation {
            BTreeNodeIntegrityValidation::Intact(validated) => {
                let expected_shape = self.registered.cell_shape();
                if validated.node().cells().iter().any(|cell| {
                    cell.key().len() != expected_shape.0
                        || (validated.node().kind() == BTreeNodeKind::Leaf
                            && cell
                                .leaf_value()
                                .is_none_or(|value| value.len() != expected_shape.1))
                }) {
                    self.damaged_node.set(Some(scope));
                    return Err(PhysicalLayoutDenial::NodeFamilyMismatch);
                }
                counters.node_decodes += 1;
                Ok(validated.node().clone())
            }
            BTreeNodeIntegrityValidation::Rejected(rejection) => {
                if matches!(&rejection, PhysicalIntegrityRejection::Damaged(_)) {
                    self.damaged_node.set(Some(scope));
                }
                Err(PhysicalLayoutDenial::NodeIntegrity(rejection))
            }
        }
    }
}

fn find_leaf_value(
    cells: &[worth_store_physical_format::BTreeNodeCellV1],
    key: &[u8],
    counters: &mut PhysicalBTreeReadCounters,
) -> Result<Option<Vec<u8>>, PhysicalLayoutDenial> {
    let mut start = 0;
    let mut end = cells.len();
    while start < end {
        let middle = start + (end - start) / 2;
        counters.key_comparisons += 1;
        match cells[middle].key().cmp(key) {
            std::cmp::Ordering::Less => start = middle + 1,
            std::cmp::Ordering::Greater => end = middle,
            std::cmp::Ordering::Equal => {
                let value = cells[middle]
                    .leaf_value()
                    .ok_or(PhysicalLayoutDenial::MalformedLeafValue)?;
                return Ok(Some(value.to_vec()));
            }
        }
    }
    Ok(None)
}

pub(super) fn decode_leaf_record(
    value: &[u8],
) -> Result<PersistedRecordIdentity, PhysicalLayoutDenial> {
    if value.len() != 24 {
        return Err(PhysicalLayoutDenial::MalformedLeafValue);
    }
    PersistedRecordIdentity::new(
        value[..16].try_into().expect("checked epoch width"),
        u64::from_le_bytes(value[16..24].try_into().expect("checked ordinal width")),
    )
    .ok_or(PhysicalLayoutDenial::MalformedLeafValue)
}

fn select_child(
    cells: &[worth_store_physical_format::BTreeNodeCellV1],
    first_child: Option<PersistedRecordIdentity>,
    key: &[u8],
    counters: &mut PhysicalBTreeReadCounters,
) -> Result<PersistedRecordIdentity, PhysicalLayoutDenial> {
    let mut child = first_child.ok_or(PhysicalLayoutDenial::NodeFamilyMismatch)?;
    for cell in cells {
        counters.key_comparisons += 1;
        if key < cell.key() {
            break;
        }
        child = cell
            .child()
            .ok_or(PhysicalLayoutDenial::NodeFamilyMismatch)?;
    }
    Ok(child)
}
