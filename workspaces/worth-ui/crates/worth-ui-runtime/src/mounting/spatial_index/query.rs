use super::{
    Bounds, Link, UiMountedSpatialBudget, UiMountedSpatialQuery, UiMountedSpatialQueryDenial,
    UiMountedSpatialWork,
};

pub(super) fn at_point(
    root: &Link,
    point: [f64; 2],
    budget: UiMountedSpatialBudget,
) -> Result<UiMountedSpatialQuery, UiMountedSpatialQueryDenial> {
    if point.iter().any(|value| !value.is_finite()) {
        return Err(UiMountedSpatialQueryDenial::InvalidGeometry);
    }
    query(root, budget, |bounds| bounds.contains(point))
}

pub(super) fn intersecting(
    root: &Link,
    bounds: [f64; 4],
    budget: UiMountedSpatialBudget,
) -> Result<UiMountedSpatialQuery, UiMountedSpatialQueryDenial> {
    let bounds = Bounds::admit(bounds).map_err(|_| UiMountedSpatialQueryDenial::InvalidGeometry)?;
    let Some(bounds) = bounds else {
        return Ok(UiMountedSpatialQuery {
            instances: Vec::new(),
            work: UiMountedSpatialWork::default(),
        });
    };
    query(root, budget, |candidate| candidate.intersects(bounds))
}

fn query(
    root: &Link,
    budget: UiMountedSpatialBudget,
    intersects: impl Fn(Bounds) -> bool,
) -> Result<UiMountedSpatialQuery, UiMountedSpatialQueryDenial> {
    let mut result = UiMountedSpatialQuery {
        instances: Vec::new(),
        work: UiMountedSpatialWork::default(),
    };
    let mut pending = Vec::new();
    if let Some(root) = root {
        pending.push(root.as_ref());
    }
    while let Some(node) = pending.pop() {
        if result.work.node_visits == budget.node_visits {
            return Err(UiMountedSpatialQueryDenial::NodeBudget { work: result.work });
        }
        result.work.node_visits += 1;
        if !intersects(node.envelope) {
            continue;
        }
        result.work.region_tests += 1;
        if intersects(node.bounds) {
            if result.instances.len() == budget.candidates {
                return Err(UiMountedSpatialQueryDenial::CandidateBudget { work: result.work });
            }
            result.instances.push(node.key.instance);
        }
        if let Some(right) = &node.right {
            pending.push(right.as_ref());
        }
        if let Some(left) = &node.left {
            pending.push(left.as_ref());
        }
    }
    result.instances.sort_unstable();
    Ok(result)
}
