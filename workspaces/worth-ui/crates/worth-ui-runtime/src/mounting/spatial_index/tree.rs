use super::{Bounds, Rc, UiMountedInstanceIdentity, UiMountedSpatialWork};

pub(super) type Link = Option<Rc<Node>>;

pub(super) struct Node {
    pub(super) key: Key,
    pub(super) bounds: Bounds,
    pub(super) envelope: Bounds,
    pub(super) left: Link,
    pub(super) right: Link,
    pub(super) height: u16,
    pub(super) len: usize,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Key {
    x: u64,
    y: u64,
    pub(super) instance: UiMountedInstanceIdentity,
}

impl Key {
    pub(super) fn new(instance: UiMountedInstanceIdentity, bounds: Bounds) -> Self {
        Self {
            x: ordered(bounds.0[0]),
            y: ordered(bounds.0[1]),
            instance,
        }
    }
}

fn ordered(value: f64) -> u64 {
    let bits = if value == 0.0 { 0.0_f64 } else { value }.to_bits();
    if bits >> 63 == 1 {
        !bits
    } else {
        bits ^ (1 << 63)
    }
}

pub(super) fn height(link: &Link) -> u16 {
    link.as_ref().map_or(0, |node| node.height)
}

pub(super) fn node(
    key: Key,
    bounds: Bounds,
    left: Link,
    right: Link,
    work: &mut UiMountedSpatialWork,
) -> Rc<Node> {
    work.node_copies += 1;
    let mut envelope = bounds;
    for child in [&left, &right].into_iter().flatten() {
        envelope = envelope.union(child.envelope);
    }
    Rc::new(Node {
        key,
        bounds,
        envelope,
        height: 1 + height(&left).max(height(&right)),
        len: 1 + left.as_ref().map_or(0, |n| n.len) + right.as_ref().map_or(0, |n| n.len),
        left,
        right,
    })
}

pub(super) fn balanced(
    key: Key,
    bounds: Bounds,
    left: Link,
    right: Link,
    work: &mut UiMountedSpatialWork,
) -> Rc<Node> {
    if height(&left) > height(&right) + 1 {
        let l = left.as_ref().unwrap();
        if height(&l.left) >= height(&l.right) {
            let r = node(key, bounds, l.right.clone(), right, work);
            return node(l.key, l.bounds, l.left.clone(), Some(r), work);
        }
        let middle = l.right.as_ref().unwrap();
        let low = node(l.key, l.bounds, l.left.clone(), middle.left.clone(), work);
        let high = node(key, bounds, middle.right.clone(), right, work);
        return node(middle.key, middle.bounds, Some(low), Some(high), work);
    }
    if height(&right) > height(&left) + 1 {
        let r = right.as_ref().unwrap();
        if height(&r.right) >= height(&r.left) {
            let l = node(key, bounds, left, r.left.clone(), work);
            return node(r.key, r.bounds, Some(l), r.right.clone(), work);
        }
        let middle = r.left.as_ref().unwrap();
        let low = node(key, bounds, left, middle.left.clone(), work);
        let high = node(r.key, r.bounds, middle.right.clone(), r.right.clone(), work);
        return node(middle.key, middle.bounds, Some(low), Some(high), work);
    }
    node(key, bounds, left, right, work)
}
