use super::tree::{balanced, node};
use super::{Bounds, Key, Link, Node, Rc, UiMountedSpatialWork};
use std::cmp::Ordering;

pub(super) fn insert(
    root: &Link,
    key: Key,
    bounds: Bounds,
    work: &mut UiMountedSpatialWork,
) -> Rc<Node> {
    let Some(root) = root else {
        return node(key, bounds, None, None, work);
    };
    work.node_visits += 1;
    match key.cmp(&root.key) {
        Ordering::Less => balanced(
            root.key,
            root.bounds,
            Some(insert(&root.left, key, bounds, work)),
            root.right.clone(),
            work,
        ),
        Ordering::Greater => balanced(
            root.key,
            root.bounds,
            root.left.clone(),
            Some(insert(&root.right, key, bounds, work)),
            work,
        ),
        Ordering::Equal => node(key, bounds, root.left.clone(), root.right.clone(), work),
    }
}

pub(super) fn remove(root: &Link, key: Key, work: &mut UiMountedSpatialWork) -> Link {
    let root = root.as_ref()?;
    work.node_visits += 1;
    match key.cmp(&root.key) {
        Ordering::Less => Some(balanced(
            root.key,
            root.bounds,
            remove(&root.left, key, work),
            root.right.clone(),
            work,
        )),
        Ordering::Greater => Some(balanced(
            root.key,
            root.bounds,
            root.left.clone(),
            remove(&root.right, key, work),
            work,
        )),
        Ordering::Equal => match (&root.left, &root.right) {
            (None, _) => root.right.clone(),
            (_, None) => root.left.clone(),
            (Some(_), Some(right)) => {
                let (successor, next_right) = remove_min(right, work);
                Some(balanced(
                    successor.key,
                    successor.bounds,
                    root.left.clone(),
                    next_right,
                    work,
                ))
            }
        },
    }
}

fn remove_min(root: &Rc<Node>, work: &mut UiMountedSpatialWork) -> (Rc<Node>, Link) {
    work.node_visits += 1;
    let Some(left) = &root.left else {
        return (Rc::clone(root), root.right.clone());
    };
    let (minimum, next_left) = remove_min(left, work);
    (
        minimum,
        Some(balanced(
            root.key,
            root.bounds,
            next_left,
            root.right.clone(),
            work,
        )),
    )
}
