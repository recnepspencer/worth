use std::rc::Rc;

use super::WorthUiPlanRegionStorageCounters;

const RADIX_WIDTH: usize = 16;
const RADIX_LEVELS: usize = 16;

#[derive(Clone, Debug)]
pub(super) enum WorthUiPlanRegionSlotSetNode {
    Branch([Option<Rc<Self>>; RADIX_WIDTH]),
    Member,
}

#[derive(Clone, Debug)]
pub(crate) struct WorthUiPlanRegionSlotSetView<const N: usize> {
    roots: [Option<Rc<WorthUiPlanRegionSlotSetNode>>; N],
    len: usize,
}

impl<const N: usize> PartialEq for WorthUiPlanRegionSlotSetView<N> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        let mut equal = true;
        self.for_each(|stable_slot| equal &= other.contains(stable_slot));
        equal
    }
}

impl<const N: usize> Eq for WorthUiPlanRegionSlotSetView<N> {}

impl<const N: usize> WorthUiPlanRegionSlotSetView<N> {
    pub(super) fn new(roots: [Option<Rc<WorthUiPlanRegionSlotSetNode>>; N], len: usize) -> Self {
        Self { roots, len }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn contains(&self, stable_slot: u64) -> bool {
        self.roots.iter().any(|root| contains(root, stable_slot))
    }

    pub(crate) fn first(&self) -> Option<u64> {
        self.roots.iter().find_map(first_slot)
    }

    pub(crate) fn for_each(&self, mut visit: impl FnMut(u64)) {
        for root in &self.roots {
            visit_slots(root, 0, &mut visit);
        }
    }
}

pub(super) fn insert(
    root: &Option<Rc<WorthUiPlanRegionSlotSetNode>>,
    stable_slot: u64,
    counters: &mut WorthUiPlanRegionStorageCounters,
) -> Option<Rc<WorthUiPlanRegionSlotSetNode>> {
    Some(insert_at(root.as_ref(), stable_slot, 0, counters))
}

pub(super) fn remove(
    root: &Option<Rc<WorthUiPlanRegionSlotSetNode>>,
    stable_slot: u64,
    counters: &mut WorthUiPlanRegionStorageCounters,
) -> Option<Rc<WorthUiPlanRegionSlotSetNode>> {
    remove_at(root.as_ref()?, stable_slot, 0, counters)
}

pub(super) fn contains(root: &Option<Rc<WorthUiPlanRegionSlotSetNode>>, stable_slot: u64) -> bool {
    let Some(mut node) = root.as_ref() else {
        return false;
    };
    for level in 0..RADIX_LEVELS {
        let WorthUiPlanRegionSlotSetNode::Branch(children) = node.as_ref() else {
            return false;
        };
        let Some(child) = children[nibble(stable_slot, level)].as_ref() else {
            return false;
        };
        node = child;
    }
    matches!(node.as_ref(), WorthUiPlanRegionSlotSetNode::Member)
}

#[cfg(test)]
pub(super) fn reachable_node_count(root: &Option<Rc<WorthUiPlanRegionSlotSetNode>>) -> usize {
    let Some(root) = root else {
        return 0;
    };
    1 + match root.as_ref() {
        WorthUiPlanRegionSlotSetNode::Branch(children) => {
            children.iter().map(reachable_node_count).sum::<usize>()
        }
        WorthUiPlanRegionSlotSetNode::Member => 0,
    }
}

fn insert_at(
    node: Option<&Rc<WorthUiPlanRegionSlotSetNode>>,
    stable_slot: u64,
    level: usize,
    counters: &mut WorthUiPlanRegionStorageCounters,
) -> Rc<WorthUiPlanRegionSlotSetNode> {
    counters.record_trie_node_copy();
    if level == RADIX_LEVELS {
        return Rc::new(WorthUiPlanRegionSlotSetNode::Member);
    }
    let mut children = match node.map(Rc::as_ref) {
        Some(WorthUiPlanRegionSlotSetNode::Branch(children)) => children.clone(),
        _ => std::array::from_fn(|_| None),
    };
    counters.record_storage_pointer_copies(RADIX_WIDTH);
    let child_index = nibble(stable_slot, level);
    children[child_index] = Some(insert_at(
        children[child_index].as_ref(),
        stable_slot,
        level + 1,
        counters,
    ));
    Rc::new(WorthUiPlanRegionSlotSetNode::Branch(children))
}

fn remove_at(
    node: &Rc<WorthUiPlanRegionSlotSetNode>,
    stable_slot: u64,
    level: usize,
    counters: &mut WorthUiPlanRegionStorageCounters,
) -> Option<Rc<WorthUiPlanRegionSlotSetNode>> {
    counters.record_trie_node_copy();
    if level == RADIX_LEVELS {
        return None;
    }
    let WorthUiPlanRegionSlotSetNode::Branch(existing) = node.as_ref() else {
        return Some(Rc::clone(node));
    };
    let child_index = nibble(stable_slot, level);
    let child = existing[child_index].as_ref()?;
    let mut children = existing.clone();
    counters.record_storage_pointer_copies(RADIX_WIDTH);
    children[child_index] = remove_at(child, stable_slot, level + 1, counters);
    children
        .iter()
        .any(Option::is_some)
        .then(|| Rc::new(WorthUiPlanRegionSlotSetNode::Branch(children)))
}

#[inline(never)]
fn visit_slots(
    root: &Option<Rc<WorthUiPlanRegionSlotSetNode>>,
    prefix: u64,
    visit: &mut dyn FnMut(u64),
) {
    let Some(root) = root else {
        return;
    };
    match root.as_ref() {
        WorthUiPlanRegionSlotSetNode::Member => visit(prefix),
        WorthUiPlanRegionSlotSetNode::Branch(children) => {
            for (index, child) in children.iter().enumerate() {
                visit_slots(child, (prefix << 4) | index as u64, visit);
            }
        }
    }
}

fn first_slot(root: &Option<Rc<WorthUiPlanRegionSlotSetNode>>) -> Option<u64> {
    let mut node = root.as_ref()?;
    let mut stable_slot = 0u64;
    loop {
        match node.as_ref() {
            WorthUiPlanRegionSlotSetNode::Member => return Some(stable_slot),
            WorthUiPlanRegionSlotSetNode::Branch(children) => {
                let (index, child) = children
                    .iter()
                    .enumerate()
                    .find_map(|(index, child)| child.as_ref().map(|child| (index, child)))?;
                stable_slot = (stable_slot << 4) | index as u64;
                node = child;
            }
        }
    }
}

fn nibble(stable_slot: u64, level: usize) -> usize {
    ((stable_slot >> ((RADIX_LEVELS - level - 1) * 4)) & 0x0f) as usize
}
