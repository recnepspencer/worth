use std::collections::{BTreeMap, BTreeSet};

use super::{Edge, PaintRow, SharedBoundary};

#[derive(Clone, Copy, Debug)]
struct Coordinate(f32);

impl PartialEq for Coordinate {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Coordinate {}

impl PartialOrd for Coordinate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Coordinate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("mounted geometry coordinates are finite")
    }
}

#[derive(Clone, Copy)]
struct BoundaryInterval {
    row: usize,
    start: Coordinate,
    end: Coordinate,
}

#[derive(Default)]
struct BoundaryBucket {
    before: Vec<BoundaryInterval>,
    after: Vec<BoundaryInterval>,
}

pub(super) fn shared_boundaries(rows: &[PaintRow]) -> (Vec<SharedBoundary>, usize) {
    let mut vertical = BTreeMap::<Coordinate, BoundaryBucket>::new();
    let mut horizontal = BTreeMap::<Coordinate, BoundaryBucket>::new();
    let mut index_rows = 0;
    for (row, paint) in rows.iter().enumerate() {
        if paint.bounds.width() == 0.0 || paint.bounds.height() == 0.0 {
            continue;
        }
        let x = Coordinate(paint.bounds.x());
        let right = Coordinate(paint.bounds.x() + paint.bounds.width());
        let y = Coordinate(paint.bounds.y());
        let bottom = Coordinate(paint.bounds.y() + paint.bounds.height());
        vertical.entry(x).or_default().after.push(BoundaryInterval {
            row,
            start: y,
            end: bottom,
        });
        vertical
            .entry(right)
            .or_default()
            .before
            .push(BoundaryInterval {
                row,
                start: y,
                end: bottom,
            });
        horizontal
            .entry(y)
            .or_default()
            .after
            .push(BoundaryInterval {
                row,
                start: x,
                end: right,
            });
        horizontal
            .entry(bottom)
            .or_default()
            .before
            .push(BoundaryInterval {
                row,
                start: x,
                end: right,
            });
        index_rows += 4;
    }
    let mut boundaries = Vec::new();
    for bucket in vertical.values() {
        join_boundary_bucket(bucket, Edge::Right, Edge::Left, &mut boundaries);
    }
    for bucket in horizontal.values() {
        join_boundary_bucket(bucket, Edge::Bottom, Edge::Top, &mut boundaries);
    }
    (boundaries, index_rows)
}

fn join_boundary_bucket(
    bucket: &BoundaryBucket,
    before_edge: Edge,
    after_edge: Edge,
    output: &mut Vec<SharedBoundary>,
) {
    let mut events = bucket
        .before
        .iter()
        .map(|interval| (interval, true))
        .chain(bucket.after.iter().map(|interval| (interval, false)))
        .collect::<Vec<_>>();
    events.sort_by_key(|(interval, before)| (interval.start, !*before, interval.row));
    let mut active_before = BTreeMap::<usize, Coordinate>::new();
    let mut active_after = BTreeMap::<usize, Coordinate>::new();
    let mut ends_before = BTreeSet::<(Coordinate, usize)>::new();
    let mut ends_after = BTreeSet::<(Coordinate, usize)>::new();
    for (interval, before) in events {
        expire_intervals(interval.start, &mut active_before, &mut ends_before);
        expire_intervals(interval.start, &mut active_after, &mut ends_after);
        let opposite = if before {
            &active_after
        } else {
            &active_before
        };
        output.extend(opposite.iter().map(|(opposite_row, opposite_end)| {
            let (before, after) = if before {
                (interval.row, *opposite_row)
            } else {
                (*opposite_row, interval.row)
            };
            SharedBoundary {
                before,
                before_edge,
                after,
                after_edge,
                start: interval.start.0,
                end: interval.end.min(*opposite_end).0,
            }
        }));
        let (active, ends) = if before {
            (&mut active_before, &mut ends_before)
        } else {
            (&mut active_after, &mut ends_after)
        };
        active.insert(interval.row, interval.end);
        ends.insert((interval.end, interval.row));
    }
}

fn expire_intervals(
    start: Coordinate,
    active: &mut BTreeMap<usize, Coordinate>,
    ends: &mut BTreeSet<(Coordinate, usize)>,
) {
    while let Some(&(end, row)) = ends.first() {
        if end > start {
            break;
        }
        ends.remove(&(end, row));
        active.remove(&row);
    }
}
