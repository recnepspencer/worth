//! AVL topology owns its node allocation; value carriers retain separate payloads.
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::storage::substrate::{StorageAllocation as Arc, StoredValue};
use std::cmp::Ordering;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
pub(super) struct MapNode<K: Ord + Copy, V: StoredValue> {
    pub(super) key: K,
    pub(super) value: V,
    pub(super) left: Option<Arc<Self>>,
    pub(super) right: Option<Arc<Self>>,
    pub(super) len: usize,
    pub(super) height: usize,
    _memory: Option<ExecutionMemoryReservation>,
}
impl<K: Ord + Copy, V: StoredValue> Clone for MapNode<K, V> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            value: self.value.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            len: self.len,
            height: self.height,
            _memory: None,
        }
    }
}
impl<K: Ord + Copy, V: StoredValue> MapNode<K, V> {
    fn allocate(
        key: K,
        value: V,
        left: Option<Arc<Self>>,
        right: Option<Arc<Self>>,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Arc<Self>, Cause> {
        let memory = crate::storage::substrate::node_memory::reserve(
            request,
            Some(Arc::<Self>::layout_bytes()),
        )?;
        let len = 1
            + left.as_ref().map_or(0, |node| node.len)
            + right.as_ref().map_or(0, |node| node.len);
        let height = 1 + height(&left).max(height(&right));
        Ok(Arc::new(Self {
            key,
            value,
            left,
            right,
            len,
            height,
            _memory: memory,
        }))
    }
    pub(super) fn from_sorted_unique(
        values: &mut impl Iterator<Item = (K, V::Value)>,
        len: usize,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Option<Arc<Self>>, Cause> {
        if len == 0 {
            return Ok(None);
        }
        if let Some(request) = request {
            request.consult()?;
        }
        let left_len = len / 2;
        let left = Self::from_sorted_unique(values, left_len, request)?;
        let (key, value) = values
            .next()
            .expect("sorted owner input has the declared length");
        let right = Self::from_sorted_unique(values, len - left_len - 1, request)?;
        Ok(Some(Self::allocate(
            key,
            V::from_value(value),
            left,
            right,
            request,
        )?))
    }
    fn detach<'a>(
        this: &'a mut Arc<Self>,
        copied: &mut u64,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<&'a mut Self, Cause> {
        if let Some(request) = request {
            request.consult()?;
        }
        if Arc::get_mut(this).is_none() {
            let memory = crate::storage::substrate::node_memory::reserve(
                request,
                Some(Arc::<Self>::layout_bytes()),
            )?;
            let node = Self {
                key: this.key,
                value: this.value.clone(),
                left: this.left.clone(),
                right: this.right.clone(),
                len: this.len,
                height: this.height,
                _memory: memory,
            };
            *copied = copied.saturating_add(std::mem::size_of::<Self>() as u64);
            *this = Arc::new(node);
        }
        Ok(Arc::get_mut(this).expect("unique actual node or newly allocated exclusive node"))
    }
    fn refresh(&mut self) {
        self.height = 1 + height(&self.left).max(height(&self.right));
        self.len = 1
            + self.left.as_ref().map_or(0, |node| node.len)
            + self.right.as_ref().map_or(0, |node| node.len);
    }
    pub(super) fn get_mut<'a>(
        root: &'a mut Option<Arc<Self>>,
        key: &K,
    ) -> Option<&'a mut V::Value> {
        let node = Arc::make_mut(root.as_mut()?);
        match key.cmp(&node.key) {
            Ordering::Less => Self::get_mut(&mut node.left, key),
            Ordering::Equal => Some(node.value.value_mut()),
            Ordering::Greater => Self::get_mut(&mut node.right, key),
        }
    }
    pub(super) fn get_mut_for_request<'a>(
        root: &'a mut Option<Arc<Self>>,
        key: &K,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<&'a mut V>, Cause>
    where
        V: StoredValue<Value = V>,
    {
        let Some(root) = root.as_mut() else {
            return Ok(None);
        };
        let node = Self::detach(root, &mut 0, Some(request))?;
        match key.cmp(&node.key) {
            Ordering::Less => Self::get_mut_for_request(&mut node.left, key, request),
            Ordering::Equal => Ok(Some(&mut node.value)),
            Ordering::Greater => Self::get_mut_for_request(&mut node.right, key, request),
        }
    }

    pub(super) fn insert(
        root: &mut Option<Arc<Self>>,
        key: K,
        value: V,
        copied: &mut u64,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Option<V>, Cause> {
        let Some(current) = root else {
            *root = Some(Self::allocate(key, value, None, None, request)?);
            return Ok(None);
        };
        let node = Self::detach(current, copied, request)?;
        let previous = match key.cmp(&node.key) {
            Ordering::Less => Self::insert(&mut node.left, key, value, copied, request)?,
            Ordering::Equal => Some(std::mem::replace(&mut node.value, value)),
            Ordering::Greater => Self::insert(&mut node.right, key, value, copied, request)?,
        };
        balance(current, copied, request)?;
        Ok(previous)
    }
    pub(super) fn remove(
        root: &mut Option<Arc<Self>>,
        key: &K,
        copied: &mut u64,
        request: Option<&ExecutionRequest<'_, '_>>,
    ) -> Result<Option<V>, Cause> {
        let Some(current) = root.as_mut() else {
            return Ok(None);
        };
        let node = Self::detach(current, copied, request)?;
        let previous = match key.cmp(&node.key) {
            Ordering::Less => Self::remove(&mut node.left, key, copied, request)?,
            Ordering::Greater => Self::remove(&mut node.right, key, copied, request)?,
            Ordering::Equal => {
                let previous = node.value.clone();
                if node.left.is_none() || node.right.is_none() {
                    *root = node.left.take().or_else(|| node.right.take());
                    return Ok(Some(previous));
                }
                let mut successor = node.right.as_deref().expect("both children exist");
                while let Some(left) = successor.left.as_deref() {
                    successor = left;
                }
                node.key = successor.key;
                node.value = successor.value.clone();
                Self::remove(&mut node.right, &node.key, copied, request)?;
                Some(previous)
            }
        };
        balance(current, copied, request)?;
        Ok(previous)
    }
    pub(super) fn into_values(root: Option<Arc<Self>>, values: &mut Vec<(K, V::Value)>) {
        let Some(current) = root else {
            return;
        };
        let node = Arc::unwrap_or_clone(current);
        Self::into_values(node.left, values);
        values.push((node.key, node.value.into_value()));
        Self::into_values(node.right, values);
    }
}
fn height<K: Ord + Copy, V: StoredValue>(node: &Option<Arc<MapNode<K, V>>>) -> usize {
    node.as_ref().map_or(0, |node| node.height)
}
fn balance<K: Ord + Copy, V: StoredValue>(
    root: &mut Arc<MapNode<K, V>>,
    copied: &mut u64,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<(), Cause> {
    MapNode::detach(root, copied, request)?.refresh();
    let delta = height(&root.left) as isize - height(&root.right) as isize;
    if delta > 1 {
        let left = root.left.as_ref().expect("AVL imbalance has a left child");
        if height(&left.left) < height(&left.right) {
            rotate_left(
                MapNode::detach(root, copied, request)?
                    .left
                    .as_mut()
                    .expect("left child"),
                copied,
                request,
            )?;
        }
        rotate_right(root, copied, request)?;
    } else if delta < -1 {
        let right = root
            .right
            .as_ref()
            .expect("AVL imbalance has a right child");
        if height(&right.right) < height(&right.left) {
            rotate_right(
                MapNode::detach(root, copied, request)?
                    .right
                    .as_mut()
                    .expect("right child"),
                copied,
                request,
            )?;
        }
        rotate_left(root, copied, request)?;
    }
    Ok(())
}
fn rotate_left<K: Ord + Copy, V: StoredValue>(
    root: &mut Arc<MapNode<K, V>>,
    copied: &mut u64,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<(), Cause> {
    let node = MapNode::detach(root, copied, request)?;
    let mut pivot = node.right.take().expect("rotation has a right child");
    node.right = MapNode::detach(&mut pivot, copied, request)?.left.take();
    node.refresh();
    MapNode::detach(&mut pivot, copied, request)?.left = Some(root.clone());
    MapNode::detach(&mut pivot, copied, request)?.refresh();
    *root = pivot;
    Ok(())
}
fn rotate_right<K: Ord + Copy, V: StoredValue>(
    root: &mut Arc<MapNode<K, V>>,
    copied: &mut u64,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<(), Cause> {
    let node = MapNode::detach(root, copied, request)?;
    let mut pivot = node.left.take().expect("rotation has a left child");
    node.left = MapNode::detach(&mut pivot, copied, request)?.right.take();
    node.refresh();
    MapNode::detach(&mut pivot, copied, request)?.right = Some(root.clone());
    MapNode::detach(&mut pivot, copied, request)?.refresh();
    *root = pivot;
    Ok(())
}
