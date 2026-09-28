use std::rc::Rc;

use super::UiPersistentIndexMutationWork;

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    value: Rc<T>,
    left: Link<T>,
    right: Link<T>,
    height: u16,
    len: usize,
}

/// Structurally shared sequence indexed by authored rank.
///
/// Insert, remove, and move copy only AVL search paths. Complete iteration is
/// reserved for initial/reconstructive materialization.
pub(crate) struct UiPersistentRankedSequence<T> {
    root: Link<T>,
}

impl<T> Clone for UiPersistentRankedSequence<T> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
        }
    }
}

impl<T> Default for UiPersistentRankedSequence<T> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<T> UiPersistentRankedSequence<T> {
    pub(crate) fn len(&self) -> usize {
        node_len(&self.root)
    }

    pub(crate) fn get(&self, mut index: usize) -> Option<&T> {
        let mut cursor = self.root.as_deref();
        while let Some(node) = cursor {
            let left = node_len(&node.left);
            match index.cmp(&left) {
                std::cmp::Ordering::Less => cursor = node.left.as_deref(),
                std::cmp::Ordering::Equal => return Some(node.value.as_ref()),
                std::cmp::Ordering::Greater => {
                    index = index.checked_sub(left + 1)?;
                    cursor = node.right.as_deref();
                }
            }
        }
        None
    }

    pub(crate) fn insert(
        &mut self,
        index: usize,
        value: T,
    ) -> Result<UiPersistentIndexMutationWork, ()> {
        if index > self.len() {
            return Err(());
        }
        let mut work = UiPersistentIndexMutationWork::default();
        self.root = Some(insert(self.root.take(), index, Rc::new(value), &mut work));
        Ok(work)
    }

    pub(crate) fn remove(
        &mut self,
        index: usize,
    ) -> Result<(Rc<T>, UiPersistentIndexMutationWork), ()> {
        if index >= self.len() {
            return Err(());
        }
        let mut work = UiPersistentIndexMutationWork::default();
        let (root, value) = remove(self.root.take(), index, &mut work);
        self.root = root;
        Ok((value.expect("validated rank exists"), work))
    }

    pub(crate) fn move_rank(
        &mut self,
        from: usize,
        to: usize,
    ) -> Result<UiPersistentIndexMutationWork, ()> {
        if from >= self.len() || to >= self.len() {
            return Err(());
        }
        if from == to {
            return Ok(UiPersistentIndexMutationWork::default());
        }
        let (value, mut work) = self.remove(from)?;
        work.merge(self.insert_rc(to, value)?)?;
        Ok(work)
    }

    fn insert_rc(
        &mut self,
        index: usize,
        value: Rc<T>,
    ) -> Result<UiPersistentIndexMutationWork, ()> {
        if index > self.len() {
            return Err(());
        }
        let mut work = UiPersistentIndexMutationWork::default();
        self.root = Some(insert(self.root.take(), index, value, &mut work));
        Ok(work)
    }

    pub(crate) fn iter(&self) -> UiPersistentRankedSequenceIter<'_, T> {
        UiPersistentRankedSequenceIter::new(
            self.root.as_deref(),
            #[cfg(test)]
            super::test_observation::observes(std::ptr::from_ref(self).cast()),
        )
    }

    pub(crate) fn retained_structural_bytes(&self) -> Option<usize> {
        let bytes_per_entry = std::mem::size_of::<Node<T>>()
            .checked_add(std::mem::size_of::<T>())?
            .checked_add(std::mem::size_of::<usize>())?;
        self.len().checked_mul(bytes_per_entry)
    }
}

fn insert<T>(
    node: Link<T>,
    index: usize,
    value: Rc<T>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<T>> {
    let Some(node) = node else {
        work.record_node_copy(true);
        return Rc::new(Node::new(value, None, None));
    };
    work.record_key_probe();
    let left_len = node_len(&node.left);
    let (left, right) = if index <= left_len {
        (
            Some(insert(node.left.clone(), index, value, work)),
            node.right.clone(),
        )
    } else {
        (
            node.left.clone(),
            Some(insert(
                node.right.clone(),
                index - left_len - 1,
                value,
                work,
            )),
        )
    };
    balance(Node::new(node.value.clone(), left, right), work)
}

fn remove<T>(
    node: Link<T>,
    index: usize,
    work: &mut UiPersistentIndexMutationWork,
) -> (Link<T>, Option<Rc<T>>) {
    let node = node.expect("validated rank has a node");
    work.record_key_probe();
    let left_len = node_len(&node.left);
    if index < left_len {
        let (left, removed) = remove(node.left.clone(), index, work);
        return (
            Some(balance(
                Node::new(node.value.clone(), left, node.right.clone()),
                work,
            )),
            removed,
        );
    }
    if index > left_len {
        let (right, removed) = remove(node.right.clone(), index - left_len - 1, work);
        return (
            Some(balance(
                Node::new(node.value.clone(), node.left.clone(), right),
                work,
            )),
            removed,
        );
    }
    let removed = Some(node.value.clone());
    match (node.left.clone(), node.right.clone()) {
        (None, right) => (right, removed),
        (left, None) => (left, removed),
        (left, Some(right)) => {
            let (right, successor) = remove_min(right, work);
            (
                Some(balance(Node::new(successor, left, right), work)),
                removed,
            )
        }
    }
}

fn remove_min<T>(node: Rc<Node<T>>, work: &mut UiPersistentIndexMutationWork) -> (Link<T>, Rc<T>) {
    work.record_key_probe();
    let Some(left) = node.left.clone() else {
        return (node.right.clone(), node.value.clone());
    };
    let (left, value) = remove_min(left, work);
    (
        Some(balance(
            Node::new(node.value.clone(), left, node.right.clone()),
            work,
        )),
        value,
    )
}

fn balance<T>(node: Node<T>, work: &mut UiPersistentIndexMutationWork) -> Rc<Node<T>> {
    work.record_node_copy(true);
    let factor = i32::from(height(&node.left)) - i32::from(height(&node.right));
    if factor > 1 {
        let left = node.left.as_ref().expect("left-heavy node has a child");
        return if height(&left.left) >= height(&left.right) {
            rotate_right(node, work)
        } else {
            rotate_right(
                Node::new(
                    node.value,
                    Some(rotate_left_rc(left.clone(), work)),
                    node.right,
                ),
                work,
            )
        };
    }
    if factor < -1 {
        let right = node.right.as_ref().expect("right-heavy node has a child");
        return if height(&right.right) >= height(&right.left) {
            rotate_left(node, work)
        } else {
            rotate_left(
                Node::new(
                    node.value,
                    node.left,
                    Some(rotate_right_rc(right.clone(), work)),
                ),
                work,
            )
        };
    }
    Rc::new(node)
}

fn rotate_left<T>(node: Node<T>, work: &mut UiPersistentIndexMutationWork) -> Rc<Node<T>> {
    let right = node.right.expect("left rotation has a right child");
    let left = Rc::new(Node::new(node.value, node.left, right.left.clone()));
    work.record_node_copy(true);
    Rc::new(Node::new(
        right.value.clone(),
        Some(left),
        right.right.clone(),
    ))
}

fn rotate_right<T>(node: Node<T>, work: &mut UiPersistentIndexMutationWork) -> Rc<Node<T>> {
    let left = node.left.expect("right rotation has a left child");
    let right = Rc::new(Node::new(node.value, left.right.clone(), node.right));
    work.record_node_copy(true);
    Rc::new(Node::new(
        left.value.clone(),
        left.left.clone(),
        Some(right),
    ))
}

fn rotate_left_rc<T>(node: Rc<Node<T>>, work: &mut UiPersistentIndexMutationWork) -> Rc<Node<T>> {
    // The inner half of a double rotation copies the child too.
    work.record_node_copy(true);
    rotate_left(
        Node::new(node.value.clone(), node.left.clone(), node.right.clone()),
        work,
    )
}

fn rotate_right_rc<T>(node: Rc<Node<T>>, work: &mut UiPersistentIndexMutationWork) -> Rc<Node<T>> {
    // The inner half of a double rotation copies the child too.
    work.record_node_copy(true);
    rotate_right(
        Node::new(node.value.clone(), node.left.clone(), node.right.clone()),
        work,
    )
}

impl<T> Node<T> {
    fn new(value: Rc<T>, left: Link<T>, right: Link<T>) -> Self {
        Self {
            value,
            height: height(&left).max(height(&right)) + 1,
            len: node_len(&left) + node_len(&right) + 1,
            left,
            right,
        }
    }
}

fn height<T>(node: &Link<T>) -> u16 {
    node.as_ref().map_or(0, |node| node.height)
}

fn node_len<T>(node: &Link<T>) -> usize {
    node.as_ref().map_or(0, |node| node.len)
}

pub(crate) struct UiPersistentRankedSequenceIter<'a, T> {
    stack: Vec<&'a Node<T>>,
    remaining: usize,
    #[cfg(test)]
    observed: bool,
}

impl<'a, T> UiPersistentRankedSequenceIter<'a, T> {
    fn new(root: Option<&'a Node<T>>, #[cfg(test)] observed: bool) -> Self {
        let mut iter = Self {
            stack: Vec::new(),
            remaining: root.map_or(0, |node| node.len),
            #[cfg(test)]
            observed,
        };
        iter.push_left(root);
        iter
    }

    fn push_left(&mut self, mut node: Option<&'a Node<T>>) {
        while let Some(current) = node {
            self.stack.push(current);
            node = current.left.as_deref();
        }
    }
}

impl<'a, T> Iterator for UiPersistentRankedSequenceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        #[cfg(test)]
        super::test_observation::observe_iteration(self.observed);
        self.remaining -= 1;
        self.push_left(node.right.as_deref());
        Some(node.value.as_ref())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for UiPersistentRankedSequenceIter<'_, T> {}

#[cfg(test)]
mod tests {
    use super::UiPersistentRankedSequence;

    #[test]
    fn ranked_churn_shares_truth_and_preserves_exact_order() {
        let mut rows = UiPersistentRankedSequence::default();
        for value in 0..4_096_u32 {
            rows.insert(rows.len(), value).unwrap();
        }
        let predecessor = rows.clone();
        rows.move_rank(2_048, 0).unwrap();
        rows.remove(10).unwrap();
        rows.insert(10, 9_999).unwrap();

        assert_eq!(predecessor.get(2_048), Some(&2_048));
        assert_eq!(rows.get(0), Some(&2_048));
        assert_eq!(rows.get(10), Some(&9_999));
        assert_eq!(rows.len(), 4_096);
        assert_eq!(rows.iter().count(), 4_096);
    }

    #[test]
    fn a_double_rotation_records_every_node_it_allocates() {
        let mut rows = UiPersistentRankedSequence::default();
        rows.insert(0, 'a').unwrap();
        rows.insert(0, 'b').unwrap();
        let work = rows.insert(1, 'c').unwrap();
        assert_eq!((work.node_copies(), work.node_allocations()), (6, 6));
        assert_eq!(rows.iter().copied().collect::<String>(), "bca");
    }
}
