use worth_store_physical_format::{BTreeNodeKind, BTreeNodeV1, PersistedRecordIdentity};

use crate::physical_runtime::{
    artifact_family::RegisteredIndexShape,
    layout::{
        PhysicalIndexPrefix, PhysicalIndexRange, PhysicalIndexScanBudget, PhysicalLayoutDenial,
    },
    LayoutPhysicalAllocation,
};

use super::lookup::{decode_leaf_record, PhysicalBTreeIndex, PhysicalBTreeReadCounters};

const MAXIMUM_HEIGHT: u8 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalBTreeRangeEntry {
    canonical_key: [u8; 24],
    selected_record: PersistedRecordIdentity,
}

impl PhysicalBTreeRangeEntry {
    pub const fn canonical_key(self) -> [u8; 24] {
        self.canonical_key
    }

    pub const fn selected_record(self) -> PersistedRecordIdentity {
        self.selected_record
    }
}

enum ScanFrame {
    Interior {
        node: BTreeNodeV1,
        next_child: usize,
    },
    Leaf {
        node: BTreeNodeV1,
        next_cell: usize,
    },
}

/// Streams one Store-selected B-tree through a bounded five-node path stack.
/// No sibling hint or whole-keyspace materialization becomes routing truth.
pub struct PhysicalBTreeRangeCursor<'index, 'access, 'runtime> {
    index: &'index PhysicalBTreeIndex<'access, 'runtime>,
    lower: [u8; 24],
    upper: Option<[u8; 24]>,
    budget: PhysicalIndexScanBudget,
    pending: Option<(PersistedRecordIdentity, Option<u8>, u8)>,
    stack: Vec<ScanFrame>,
    counters: PhysicalBTreeReadCounters,
    done: bool,
    _allocation: LayoutPhysicalAllocation<'runtime>,
}

impl<'access, 'runtime> PhysicalBTreeIndex<'access, 'runtime> {
    pub fn range<'index>(
        &'index self,
        range: PhysicalIndexRange,
        budget: PhysicalIndexScanBudget,
    ) -> Result<PhysicalBTreeRangeCursor<'index, 'access, 'runtime>, PhysicalLayoutDenial> {
        self.damaged_node.set(None);
        self.registered
            .admit_shape(RegisteredIndexShape::Range)
            .map_err(|_| PhysicalLayoutDenial::ShapeNotAdmitted(self.registered.family()))?;
        self.admit_scan(
            range.family(),
            range.store(),
            range.lower(),
            range.upper(),
            budget,
        )
    }

    pub fn prefix<'index>(
        &'index self,
        prefix: PhysicalIndexPrefix,
        budget: PhysicalIndexScanBudget,
    ) -> Result<PhysicalBTreeRangeCursor<'index, 'access, 'runtime>, PhysicalLayoutDenial> {
        self.damaged_node.set(None);
        self.registered
            .admit_shape(RegisteredIndexShape::Prefix)
            .map_err(|_| PhysicalLayoutDenial::ShapeNotAdmitted(self.registered.family()))?;
        let (lower, upper) = prefix.bounds();
        self.admit_scan(prefix.family(), prefix.store(), lower, upper, budget)
    }

    fn admit_scan<'index>(
        &'index self,
        family: worth_store_contracts::DurableArtifactFamilyId,
        store: worth_store_physical_format::store_namespace::StableStoreIdentity,
        lower: [u8; 24],
        upper: Option<[u8; 24]>,
        budget: PhysicalIndexScanBudget,
    ) -> Result<PhysicalBTreeRangeCursor<'index, 'access, 'runtime>, PhysicalLayoutDenial> {
        if family != self.registered.family() {
            return Err(PhysicalLayoutDenial::ShapeNotAdmitted(
                self.registered.family(),
            ));
        }
        if store != self.port.reader().store_identity() {
            return Err(PhysicalLayoutDenial::ForeignStore);
        }
        let allocation = self
            .port
            .admit_scan_stack()
            .map_err(PhysicalLayoutDenial::ScanScratchUnavailable)?;
        Ok(PhysicalBTreeRangeCursor {
            index: self,
            lower,
            upper,
            budget,
            pending: self.root.map(|root| (root, None, 0)),
            stack: Vec::with_capacity(MAXIMUM_HEIGHT as usize),
            counters: PhysicalBTreeReadCounters::default(),
            done: self.root.is_none(),
            _allocation: allocation,
        })
    }
}

impl PhysicalBTreeRangeCursor<'_, '_, '_> {
    pub const fn counters(&self) -> PhysicalBTreeReadCounters {
        self.counters
    }

    fn load_pending(&mut self) -> Result<(), PhysicalLayoutDenial> {
        let Some((record, expected_level, depth)) = self.pending.take() else {
            return Ok(());
        };
        if depth >= MAXIMUM_HEIGHT {
            return Err(PhysicalLayoutDenial::TreeDepthExceeded);
        }
        if self.counters.page_touches() >= self.budget.maximum_pages() {
            return Err(PhysicalLayoutDenial::ScanBudgetExhausted);
        }
        let node = self.index.read_selected_node(record, &mut self.counters)?;
        if self.counters.page_touches() > self.budget.maximum_pages() {
            return Err(PhysicalLayoutDenial::ScanBudgetExhausted);
        }
        if expected_level.is_some_and(|level| node.level() != level)
            || node.cells().iter().any(|cell| cell.key().len() != 24)
        {
            return Err(PhysicalLayoutDenial::TreeTopologyDamaged);
        }
        match node.kind() {
            BTreeNodeKind::Interior => {
                let lower = self.lower;
                let next_child = node.cells().partition_point(|cell| {
                    self.counters.observe_comparison();
                    cell.key() <= lower.as_slice()
                });
                self.stack.push(ScanFrame::Interior { node, next_child });
            }
            BTreeNodeKind::Leaf => {
                let lower = self.lower;
                let next_cell = node.cells().partition_point(|cell| {
                    self.counters.observe_comparison();
                    cell.key() < lower.as_slice()
                });
                self.stack.push(ScanFrame::Leaf { node, next_cell });
            }
        }
        Ok(())
    }

    fn advance(&mut self) -> Result<Option<PhysicalBTreeRangeEntry>, PhysicalLayoutDenial> {
        loop {
            if self.pending.is_some() {
                self.load_pending()?;
                continue;
            }
            let next_depth = self.stack.len() as u8;
            let Some(frame) = self.stack.last_mut() else {
                self.done = true;
                return Ok(None);
            };
            match frame {
                ScanFrame::Leaf { node, next_cell } => {
                    let Some(cell) = node.cells().get(*next_cell) else {
                        self.stack.pop();
                        continue;
                    };
                    *next_cell += 1;
                    self.counters.observe_comparison();
                    if self
                        .upper
                        .is_some_and(|upper| cell.key() >= upper.as_slice())
                    {
                        self.stack.pop();
                        continue;
                    }
                    let canonical_key: [u8; 24] = cell.key().try_into().expect("checked key width");
                    let selected_record = decode_leaf_record(
                        cell.leaf_value()
                            .ok_or(PhysicalLayoutDenial::MalformedLeafValue)?,
                    )?;
                    return Ok(Some(PhysicalBTreeRangeEntry {
                        canonical_key,
                        selected_record,
                    }));
                }
                ScanFrame::Interior { node, next_child } => {
                    let position = *next_child;
                    if position > node.cells().len() {
                        self.stack.pop();
                        continue;
                    }
                    *next_child += 1;
                    if position > 0 {
                        self.counters.observe_comparison();
                        if self.upper.is_some_and(|upper| {
                            node.cells()[position - 1].key() >= upper.as_slice()
                        }) {
                            self.stack.pop();
                            continue;
                        }
                    }
                    let child = if position == 0 {
                        node.first_child()
                    } else {
                        node.cells()[position - 1].child()
                    }
                    .ok_or(PhysicalLayoutDenial::TreeTopologyDamaged)?;
                    self.pending = Some((child, Some(node.level() - 1), next_depth));
                }
            }
        }
    }
}

impl Iterator for PhysicalBTreeRangeCursor<'_, '_, '_> {
    type Item = Result<PhysicalBTreeRangeEntry, PhysicalLayoutDenial>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.advance() {
            Ok(Some(entry)) => Some(Ok(entry)),
            Ok(None) => None,
            Err(denial) => {
                self.done = true;
                Some(Err(denial))
            }
        }
    }
}
