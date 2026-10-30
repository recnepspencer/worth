use super::record::UiSpatialRecord;

#[derive(Clone, Copy)]
struct UiIntervalNode {
    entry: usize,
    left: Option<usize>,
    right: Option<usize>,
    subtree_max_right: u32,
}

pub(crate) struct UiImmutableIntervalIndex<Record> {
    records: Box<[Record]>,
    nodes: Box<[UiIntervalNode]>,
    root: Option<usize>,
}

pub(crate) struct UiBoundedPointCandidates<'index, Record> {
    records: Vec<&'index Record>,
    probes: usize,
    exhausted: bool,
}

pub(crate) struct UiBoundedRegionCandidates<'index, Record> {
    records: Vec<&'index Record>,
    probes: usize,
    exhausted: bool,
}

struct UiPointTraversal<'index, Record> {
    point: worth_ui_inspection::UiClientPhysicalPixel,
    maximum_candidates: usize,
    candidates: Vec<&'index Record>,
    probes: usize,
}

struct UiRegionTraversal<'index, Record> {
    region: worth_ui_inspection::UiClientPhysicalRect,
    maximum_candidates: usize,
    candidates: Vec<&'index Record>,
    probes: usize,
}

impl<Record: UiSpatialRecord> UiImmutableIntervalIndex<Record> {
    pub(crate) fn build(records: Vec<Record>) -> Self {
        let mut order = (0..records.len()).collect::<Vec<_>>();
        order.sort_unstable_by_key(|index| records[*index].region().left());
        let mut nodes = Vec::with_capacity(records.len());
        let root = build_balanced(&records, &order, &mut nodes);
        Self {
            records: records.into_boxed_slice(),
            nodes: nodes.into_boxed_slice(),
            root,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }

    pub(crate) fn records(&self) -> &[Record] {
        &self.records
    }

    pub(crate) fn structural_digest(&self) -> u64 {
        let mut digest = self.root.map_or(0, |root| root as u64).rotate_left(7);
        for node in &self.nodes {
            let record = &self.records[node.entry];
            for value in [
                node.entry as u64,
                node.left.map_or(u64::MAX, |index| index as u64),
                node.right.map_or(u64::MAX, |index| index as u64),
                u64::from(node.subtree_max_right),
                u64::from(record.region().left()),
                u64::from(record.region().right()),
                record.semantic_digest(),
            ] {
                digest = (digest ^ value).wrapping_mul(0x100000001b3);
            }
        }
        digest
    }

    pub(crate) fn point_candidates(
        &self,
        point: worth_ui_inspection::UiClientPhysicalPixel,
        maximum_candidates: usize,
    ) -> UiBoundedPointCandidates<'_, Record> {
        let mut traversal = UiPointTraversal {
            point,
            maximum_candidates,
            candidates: Vec::new(),
            probes: 0,
        };
        let exhausted = self.collect_point(self.root, &mut traversal);
        UiBoundedPointCandidates {
            records: traversal.candidates,
            probes: traversal.probes,
            exhausted,
        }
    }

    fn collect_point<'index>(
        &'index self,
        node: Option<usize>,
        traversal: &mut UiPointTraversal<'index, Record>,
    ) -> bool {
        let Some(node_index) = node else {
            return false;
        };
        let node = self.nodes[node_index];
        traversal.probes += 1;
        if let Some(left) = node
            .left
            .filter(|left| self.nodes[*left].subtree_max_right > traversal.point.x())
            && self.collect_point(Some(left), traversal)
        {
            return true;
        }
        let record = &self.records[node.entry];
        if record.region().contains(traversal.point) {
            if traversal.candidates.len() == traversal.maximum_candidates {
                return true;
            }
            traversal.candidates.push(record);
        }
        if record.region().left() <= traversal.point.x() {
            return self.collect_point(node.right, traversal);
        }
        false
    }

    pub(crate) fn region_candidates(
        &self,
        region: worth_ui_inspection::UiClientPhysicalRect,
        maximum_candidates: usize,
    ) -> UiBoundedRegionCandidates<'_, Record> {
        let mut traversal = UiRegionTraversal {
            region,
            maximum_candidates,
            candidates: Vec::new(),
            probes: 0,
        };
        let exhausted = self.collect_region(self.root, &mut traversal);
        UiBoundedRegionCandidates {
            records: traversal.candidates,
            probes: traversal.probes,
            exhausted,
        }
    }

    fn collect_region<'index>(
        &'index self,
        node: Option<usize>,
        traversal: &mut UiRegionTraversal<'index, Record>,
    ) -> bool {
        let Some(node_index) = node else {
            return false;
        };
        let node = self.nodes[node_index];
        traversal.probes += 1;
        if let Some(left) = node
            .left
            .filter(|left| self.nodes[*left].subtree_max_right > traversal.region.left())
            && self.collect_region(Some(left), traversal)
        {
            return true;
        }
        let record = &self.records[node.entry];
        if record.region().intersects(traversal.region) {
            if traversal.candidates.len() == traversal.maximum_candidates {
                return true;
            }
            traversal.candidates.push(record);
        }
        if record.region().left() < traversal.region.right() {
            return self.collect_region(node.right, traversal);
        }
        false
    }
}

pub(super) fn estimated_retained_structural_bytes<Record>(record_count: usize) -> Option<usize> {
    record_count
        .checked_mul(std::mem::size_of::<Record>())?
        .checked_add(record_count.checked_mul(std::mem::size_of::<UiIntervalNode>())?)
}

impl<'index, Record> UiBoundedPointCandidates<'index, Record> {
    pub(crate) fn into_parts(self) -> (Vec<&'index Record>, usize, bool) {
        (self.records, self.probes, self.exhausted)
    }
}

impl<'index, Record> UiBoundedRegionCandidates<'index, Record> {
    pub(crate) fn into_parts(self) -> (Vec<&'index Record>, usize, bool) {
        (self.records, self.probes, self.exhausted)
    }
}

fn build_balanced<Record: UiSpatialRecord>(
    records: &[Record],
    order: &[usize],
    nodes: &mut Vec<UiIntervalNode>,
) -> Option<usize> {
    let (entry, left_order, right_order) = split_middle(order)?;
    let node_index = nodes.len();
    nodes.push(UiIntervalNode {
        entry,
        left: None,
        right: None,
        subtree_max_right: records[entry].region().right(),
    });
    let left = build_balanced(records, left_order, nodes);
    let right = build_balanced(records, right_order, nodes);
    let subtree_max_right = [
        Some(records[entry].region().right()),
        left.map(|i| nodes[i].subtree_max_right),
        right.map(|i| nodes[i].subtree_max_right),
    ]
    .into_iter()
    .flatten()
    .max()
    .expect("one entry always contributes a right edge");
    nodes[node_index] = UiIntervalNode {
        entry,
        left,
        right,
        subtree_max_right,
    };
    Some(node_index)
}

fn split_middle(order: &[usize]) -> Option<(usize, &[usize], &[usize])> {
    (!order.is_empty()).then(|| {
        let middle = order.len() / 2;
        (order[middle], &order[..middle], &order[middle + 1..])
    })
}
