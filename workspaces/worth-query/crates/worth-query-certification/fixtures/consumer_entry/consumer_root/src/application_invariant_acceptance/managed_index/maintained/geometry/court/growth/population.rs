//! Independent interpreter of the authored chain/sharing inputs and coordinates.
use super::*;
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum SharedChain {
    MiddlePair,
    All,
}
pub(super) type EdgeValue = (String, Coordinates, Coordinates);
pub(super) fn universe(
    directory: &str,
    vertices: u32,
    unrelated: u32,
    moved: Option<i64>,
    shared: SharedChain,
) -> Vec<EdgeValue> {
    let point = |x| Coordinates::new(x, 0, x, 0);
    let middle = vertices / 2;
    let mut rows = expected(directory, 10, all(directory).probe);
    for index in 0..vertices - 1 {
        if directory == "B"
            && shared == SharedChain::MiddlePair
            && index != middle - 1
            && index != middle
        {
            continue;
        }
        let endpoint = |vertex| {
            if vertex == middle {
                moved.unwrap_or(1_000 + i64::from(vertex) * 2)
            } else {
                1_000 + i64::from(vertex) * 2
            }
        };
        rows.push((
            format!("chain-e-{index}"),
            point(endpoint(index)),
            point(endpoint(index + 1)),
        ));
    }
    for index in 0..unrelated {
        let batch = index / REMOTE_BATCH_EDGES;
        let offset = index % REMOTE_BATCH_EDGES;
        let start = 1_000_000 + i64::from(batch * REMOTE_BATCH_EDGES) * 4 + i64::from(offset) * 2;
        rows.push((
            format!("remote-{batch}-e-{offset}"),
            point(start),
            point(start + 2),
        ));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}
pub(super) fn contacts(
    directory: &str,
    vertices: u32,
    unrelated: u32,
    moved: Option<i64>,
    shared: SharedChain,
) -> Vec<EdgeValue> {
    intersecting(universe(directory, vertices, unrelated, moved, shared), 50)
}
pub(super) fn intersecting(rows: Vec<EdgeValue>, x: i64) -> Vec<EdgeValue> {
    rows.into_iter()
        .filter(|(_, a, b)| {
            a.lower_x.min(b.lower_x) <= x
                && x <= a.upper_x.max(b.upper_x)
                && a.lower_y.min(b.lower_y) <= 0
                && 0 <= a.upper_y.max(b.upper_y)
        })
        .collect()
}
