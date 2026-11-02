//! Closed linear geometry in one exact typed integer unit/frame domain.
//! This fixture predicate supplies no graph, topology, or publication authority.
#[cfg(test)]
use crate::Box2;
use crate::{integer_orientation as orientation, BoxSpace, IntegerBox2, PlanarInteger};
use std::cmp::Ordering;
#[cfg(test)]
mod tests;
type Point<C = i64> = [C; 2];

/// Endpoint regions wider than a point do not specify a unique segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SegmentBoxDenial {
    NonPointEndpoint,
}
impl std::fmt::Display for SegmentBoxDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("exact segment requires point endpoints")
    }
}
impl std::error::Error for SegmentBoxDenial {}
/// Admits the fixture's genuine endpoint regions before interpreting a segment.
pub fn endpoint_segment_intersects_box<Space: BoxSpace>(
    start: IntegerBox2<Space>,
    end: IntegerBox2<Space>,
    query: IntegerBox2<Space>,
) -> Result<bool, SegmentBoxDenial> {
    let start = start.coordinates();
    let end = end.coordinates();
    if start.lower != start.upper || end.lower != end.upper {
        return Err(SegmentBoxDenial::NonPointEndpoint);
    }
    Ok(segment_intersects_box(start.lower, end.lower, query))
}

/// Exact contact of a closed segment and closed box, including degeneracies.
/// Points have the same declared unit/frame as the query box. All arithmetic
/// has fixed width and bounded work; no tolerance or floating projection enters.
pub fn segment_intersects_box<Space: BoxSpace>(
    start: Point<Space::Coordinate>,
    end: Point<Space::Coordinate>,
    query: IntegerBox2<Space>,
) -> bool {
    let bounds = query.coordinates();
    let inside = |point: Point<Space::Coordinate>| {
        (0..2).all(|axis| bounds.lower[axis] <= point[axis] && point[axis] <= bounds.upper[axis])
    };
    if inside(start) || inside(end) {
        return true;
    }
    let [left, bottom] = bounds.lower;
    let [right, top] = bounds.upper;
    let corners = [[left, bottom], [right, bottom], [right, top], [left, top]];
    (0..4).any(|edge| segments_intersect(start, end, corners[edge], corners[(edge + 1) % 4]))
}
fn segments_intersect<C: PlanarInteger + Ord>(
    a: Point<C>,
    b: Point<C>,
    c: Point<C>,
    d: Point<C>,
) -> bool {
    let ab_c = orientation(a, b, c);
    let ab_d = orientation(a, b, d);
    let cd_a = orientation(c, d, a);
    let cd_b = orientation(c, d, b);
    if (ab_c == Ordering::Equal && on_segment(a, b, c))
        || (ab_d == Ordering::Equal && on_segment(a, b, d))
        || (cd_a == Ordering::Equal && on_segment(c, d, a))
        || (cd_b == Ordering::Equal && on_segment(c, d, b))
    {
        return true;
    }
    opposite(ab_c, ab_d) && opposite(cd_a, cd_b)
}
fn opposite(a: Ordering, b: Ordering) -> bool {
    matches!(
        (a, b),
        (Ordering::Less, Ordering::Greater) | (Ordering::Greater, Ordering::Less)
    )
}
fn on_segment<C: Copy + Ord>(a: Point<C>, b: Point<C>, point: Point<C>) -> bool {
    (0..2).all(|axis| a[axis].min(b[axis]) <= point[axis] && point[axis] <= a[axis].max(b[axis]))
}
