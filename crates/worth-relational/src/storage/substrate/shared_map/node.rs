use crate::storage::substrate::StorageAllocation as Arc;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub(super) struct MapNode<K: Ord + Copy, V: Clone> {
    pub(super) key: K,
    pub(super) value: Arc<V>,
    pub(super) left: Option<Arc<Self>>,
    pub(super) right: Option<Arc<Self>>,
    pub(super) len: usize,
    pub(super) height: usize,
}

impl<K: Ord + Copy, V: Clone> MapNode<K, V> {
    pub(super) fn from_sorted_unique(
        values: &mut impl Iterator<Item = (K, V)>,
        len: usize,
    ) -> Option<Arc<Self>> {
        if len == 0 {
            return None;
        }
        let left_len = len / 2;
        let left = Self::from_sorted_unique(values, left_len);
        let (key, value) = values.next().expect("sorted input length is exact");
        let right = Self::from_sorted_unique(values, len - left_len - 1);
        let height = 1 + height(&left).max(height(&right));
        Some(Arc::new(Self {
            key,
            value: Arc::new(value),
            left,
            right,
            len,
            height,
        }))
    }

    fn detach<'a>(this: &'a mut Arc<Self>, copied_bytes: &mut u64) -> &'a mut Self {
        if !this.is_unique() {
            *copied_bytes = copied_bytes.saturating_add(std::mem::size_of::<Self>() as u64);
        }
        Arc::make_mut(this)
    }

    fn refresh(&mut self) {
        self.height = 1 + height(&self.left).max(height(&self.right));
        self.len = 1
            + self.left.as_ref().map_or(0, |node| node.len)
            + self.right.as_ref().map_or(0, |node| node.len);
    }

    pub(super) fn get_mut<'a>(root: &'a mut Option<Arc<Self>>, key: &K) -> Option<&'a mut V> {
        let node = Arc::make_mut(root.as_mut()?);
        match key.cmp(&node.key) {
            Ordering::Less => Self::get_mut(&mut node.left, key),
            Ordering::Equal => Some(Arc::make_mut(&mut node.value)),
            Ordering::Greater => Self::get_mut(&mut node.right, key),
        }
    }

    pub(super) fn insert(
        root: &mut Option<Arc<Self>>,
        key: K,
        value: Arc<V>,
        copied_bytes: &mut u64,
    ) -> Option<Arc<V>> {
        let Some(current) = root else {
            *root = Some(Arc::new(Self {
                key,
                value,
                left: None,
                right: None,
                len: 1,
                height: 1,
            }));
            return None;
        };
        let node = Self::detach(current, copied_bytes);
        let previous = match key.cmp(&node.key) {
            Ordering::Less => Self::insert(&mut node.left, key, value, copied_bytes),
            Ordering::Equal => Some(std::mem::replace(&mut node.value, value)),
            Ordering::Greater => Self::insert(&mut node.right, key, value, copied_bytes),
        };
        balance(current, copied_bytes);
        previous
    }

    pub(super) fn remove(
        root: &mut Option<Arc<Self>>,
        key: &K,
        copied_bytes: &mut u64,
    ) -> Option<Arc<V>> {
        let current = root.as_mut()?;
        let node = Self::detach(current, copied_bytes);
        let previous = match key.cmp(&node.key) {
            Ordering::Less => Self::remove(&mut node.left, key, copied_bytes),
            Ordering::Greater => Self::remove(&mut node.right, key, copied_bytes),
            Ordering::Equal => {
                let previous = node.value.clone();
                if node.left.is_none() || node.right.is_none() {
                    *root = node.left.take().or_else(|| node.right.take());
                    return Some(previous);
                }
                let mut successor = node.right.as_deref().unwrap();
                while let Some(left) = successor.left.as_deref() {
                    successor = left;
                }
                node.key = successor.key;
                node.value = successor.value.clone();
                Self::remove(&mut node.right, &node.key, copied_bytes);
                Some(previous)
            }
        };
        balance(current, copied_bytes);
        previous
    }

    pub(super) fn collect_mut<'a>(
        root: &'a mut Option<Arc<Self>>,
        values: &mut Vec<(&'a K, &'a mut V)>,
    ) {
        let Some(current) = root else {
            return;
        };
        let node = Arc::make_mut(current);
        Self::collect_mut(&mut node.left, values);
        values.push((&node.key, Arc::make_mut(&mut node.value)));
        Self::collect_mut(&mut node.right, values);
    }

    pub(super) fn into_values(root: Option<Arc<Self>>, values: &mut Vec<(K, V)>) {
        let Some(current) = root else {
            return;
        };
        let node = Arc::unwrap_or_clone(current);
        Self::into_values(node.left, values);
        values.push((node.key, Arc::unwrap_or_clone(node.value)));
        Self::into_values(node.right, values);
    }
}

fn height<K: Ord + Copy, V: Clone>(node: &Option<Arc<MapNode<K, V>>>) -> usize {
    node.as_ref().map_or(0, |node| node.height)
}

fn balance<K: Ord + Copy, V: Clone>(root: &mut Arc<MapNode<K, V>>, copied_bytes: &mut u64) {
    MapNode::detach(root, copied_bytes).refresh();
    let delta = height(&root.left) as isize - height(&root.right) as isize;
    if delta > 1 {
        let left = root.left.as_ref().unwrap();
        if height(&left.left) < height(&left.right) {
            rotate_left(
                MapNode::detach(root, copied_bytes).left.as_mut().unwrap(),
                copied_bytes,
            );
        }
        rotate_right(root, copied_bytes);
    } else if delta < -1 {
        let right = root.right.as_ref().unwrap();
        if height(&right.right) < height(&right.left) {
            rotate_right(
                MapNode::detach(root, copied_bytes).right.as_mut().unwrap(),
                copied_bytes,
            );
        }
        rotate_left(root, copied_bytes);
    }
}

fn rotate_left<K: Ord + Copy, V: Clone>(root: &mut Arc<MapNode<K, V>>, copied_bytes: &mut u64) {
    let node = MapNode::detach(root, copied_bytes);
    let mut pivot = node.right.take().unwrap();
    node.right = MapNode::detach(&mut pivot, copied_bytes).left.take();
    node.refresh();
    MapNode::detach(&mut pivot, copied_bytes).left = Some(root.clone());
    MapNode::detach(&mut pivot, copied_bytes).refresh();
    *root = pivot;
}

fn rotate_right<K: Ord + Copy, V: Clone>(root: &mut Arc<MapNode<K, V>>, copied_bytes: &mut u64) {
    let node = MapNode::detach(root, copied_bytes);
    let mut pivot = node.left.take().unwrap();
    node.left = MapNode::detach(&mut pivot, copied_bytes).right.take();
    node.refresh();
    MapNode::detach(&mut pivot, copied_bytes).right = Some(root.clone());
    MapNode::detach(&mut pivot, copied_bytes).refresh();
    *root = pivot;
}
