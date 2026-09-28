use std::rc::Rc;

const RADIX_BITS: usize = 4;
const RADIX: usize = 1 << RADIX_BITS;
const RADIX_MASK: u32 = (1 << RADIX_BITS) - 1;
const LEVELS: usize = u32::BITS as usize / RADIX_BITS;

pub(crate) struct UiPersistentSlotTrie<V> {
    root: Option<Rc<Branch<V>>>,
    len: usize,
}

struct Branch<V> {
    children: [Option<Rc<Node<V>>>; RADIX],
}

enum Node<V> {
    Branch(Branch<V>),
    Values([Option<Rc<V>>; RADIX]),
}

impl<V> Clone for UiPersistentSlotTrie<V> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            len: self.len,
        }
    }
}

impl<V> Default for UiPersistentSlotTrie<V> {
    fn default() -> Self {
        Self { root: None, len: 0 }
    }
}

impl<V> UiPersistentSlotTrie<V> {
    pub(crate) fn get(&self, slot: usize) -> Option<&V> {
        let slot = u32::try_from(slot).ok()?;
        let mut branch = self.root.as_deref()?;
        for level in (2..LEVELS).rev() {
            let node = branch.children[nibble(slot, level)].as_deref()?;
            let Node::Branch(next) = node else {
                return None;
            };
            branch = next;
        }
        let node = branch.children[nibble(slot, 1)].as_deref()?;
        let Node::Values(values) = node else {
            return None;
        };
        values[nibble(slot, 0)].as_deref()
    }

    pub(crate) fn insert(&mut self, slot: usize, value: V) -> bool {
        let slot = u32::try_from(slot).expect("admitted compact slot fits u32");
        let (root, replaced) =
            insert_branch(self.root.as_deref(), slot, LEVELS - 1, Rc::new(value));
        self.root = Some(Rc::new(root));
        if !replaced {
            self.len = self
                .len
                .checked_add(1)
                .expect("compact slot count exhausted");
        }
        replaced
    }

    #[cfg(test)]
    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn retained_structural_bytes(&self) -> Option<usize> {
        let branch_bytes = LEVELS.checked_mul(std::mem::size_of::<Branch<V>>())?;
        let value_bytes = std::mem::size_of::<V>()
            .checked_add(2usize.checked_mul(std::mem::size_of::<usize>())?)?;
        self.len.checked_mul(branch_bytes.checked_add(value_bytes)?)
    }
}

fn insert_branch<V>(
    current: Option<&Branch<V>>,
    slot: u32,
    level: usize,
    value: Rc<V>,
) -> (Branch<V>, bool) {
    let mut children = current
        .map(|branch| branch.children.clone())
        .unwrap_or_else(|| std::array::from_fn(|_| None));
    let index = nibble(slot, level);
    if level == 1 {
        let mut values = match children[index].as_deref() {
            Some(Node::Values(values)) => values.clone(),
            Some(Node::Branch(_)) | None => std::array::from_fn(|_| None),
        };
        let value_index = nibble(slot, 0);
        let replaced = values[value_index].replace(value).is_some();
        children[index] = Some(Rc::new(Node::Values(values)));
        return (Branch { children }, replaced);
    }
    let child = match children[index].as_deref() {
        Some(Node::Branch(branch)) => Some(branch),
        Some(Node::Values(_)) | None => None,
    };
    let (next, replaced) = insert_branch(child, slot, level - 1, value);
    children[index] = Some(Rc::new(Node::Branch(next)));
    (Branch { children }, replaced)
}

const fn nibble(slot: u32, level: usize) -> usize {
    ((slot >> (level * RADIX_BITS)) & RADIX_MASK) as usize
}

#[cfg(test)]
mod tests {
    use super::UiPersistentSlotTrie;

    #[test]
    fn exact_slots_fork_without_mutating_the_predecessor() {
        let mut current = UiPersistentSlotTrie::default();
        assert!(!current.insert(0, "zero"));
        assert!(!current.insert(65_535, "last"));
        let predecessor = current.clone();

        assert!(current.insert(0, "updated"));
        assert!(!current.insert(17, "seventeen"));

        assert_eq!(predecessor.get(0), Some(&"zero"));
        assert_eq!(predecessor.get(17), None);
        assert_eq!(current.get(0), Some(&"updated"));
        assert_eq!(current.get(17), Some(&"seventeen"));
        assert_eq!(current.get(65_535), Some(&"last"));
        assert_eq!(predecessor.len(), 2);
        assert_eq!(current.len(), 3);
    }
}
