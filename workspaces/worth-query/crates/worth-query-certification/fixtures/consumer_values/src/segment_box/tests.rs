//! Independent bounded rational clipping and literal full-width contacts.
use super::*;
use crate::Box2Coordinates;
fn query(lower: Point, upper: Point) -> Box2 {
    Box2::new(Box2Coordinates { lower, upper }).unwrap()
}

// A different algorithm: intersect parameter slabs using exact SMALL i128
// rational comparisons. The enumerated coordinates bound every product by64.
fn clipping_oracle(a: Point, b: Point, bounds: Box2Coordinates) -> bool {
    let compare = |a: (i128, i128), b: (i128, i128)| (a.0 * b.1).cmp(&(b.0 * a.1));
    let ratio = |n: i128, d: i128| if d < 0 { (-n, -d) } else { (n, d) };
    let mut low = (0, 1);
    let mut high = (1, 1);
    for axis in 0..2 {
        let origin = i128::from(a[axis]);
        let direction = i128::from(b[axis]) - origin;
        if direction == 0 {
            if a[axis] < bounds.lower[axis] || a[axis] > bounds.upper[axis] {
                return false;
            }
            continue;
        }
        let mut entry = ratio(i128::from(bounds.lower[axis]) - origin, direction);
        let mut exit = ratio(i128::from(bounds.upper[axis]) - origin, direction);
        if compare(entry, exit).is_gt() {
            std::mem::swap(&mut entry, &mut exit);
        }
        if compare(entry, low).is_gt() {
            low = entry;
        }
        if compare(exit, high).is_lt() {
            high = exit;
        }
        if compare(low, high).is_gt() {
            return false;
        }
    }
    true
}
#[test]
fn closed_segment_box_matches_independent_parameter_clipping_and_enclosure_law() {
    let probes = [
        query([0, 0], [0, 0]),
        query([-1, -1], [1, 1]),
        query([0, -2], [0, 2]),
        query([-2, 0], [2, 0]),
        query([1, 1], [2, 2]),
        query([-2, -2], [-1, -1]),
        query([-2, -1], [2, 1]),
        query([2, -2], [2, 2]),
        query([-2, 2], [2, 2]),
    ];
    for ax in -2..=2 {
        for ay in -2..=2 {
            for bx in -2..=2 {
                for by in -2..=2 {
                    let start = [ax, ay];
                    let end = [bx, by];
                    for probe in probes {
                        let actual = segment_intersects_box(start, end, probe);
                        assert_eq!(
                            actual,
                            clipping_oracle(start, end, probe.coordinates()),
                            "{start:?}->{end:?}, {probe:?}"
                        );
                        assert_eq!(actual, segment_intersects_box(end, start, probe));
                        let bounds = probe.coordinates();
                        let candidate = (0..2).all(|axis| {
                            start[axis].min(end[axis]) <= bounds.upper[axis]
                                && bounds.lower[axis] <= start[axis].max(end[axis])
                        });
                        assert!(
                            !actual || candidate,
                            "an exact contact cannot be pruned by its AABB candidate"
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn closed_segment_box_full_i64_and_degenerate_cases_keep_exact_boundary_truth() {
    let low = i64::MIN;
    let high = i64::MAX;
    for (start, end, point, expected) in [
        ([low, low], [high, high], [0, 0], true),
        ([low, low], [high, high], [0, 1], false),
        ([low, high], [high, low], [0, -1], true),
        ([low, high], [high, low], [0, 0], false),
        ([low, low], [high, low], [high, low], true),
        ([low, low], [high, low], [high, low + 1], false),
        ([low, high], [low, high], [low, high], true),
        ([low, high], [low, high], [high, low], false),
    ] {
        assert_eq!(
            segment_intersects_box(start, end, query(point, point)),
            expected
        );
        assert_eq!(
            segment_intersects_box(end, start, query(point, point)),
            expected
        );
        assert!(segment_intersects_box(
            start,
            end,
            query([low, low], [high, high])
        ));
    }
    assert!(
        !segment_intersects_box([0, 0], [10, 10], query([4, 5], [4, 5])),
        "a genuine AABB hit can fail exact contact"
    );
    assert!(segment_intersects_box(
        [0, 0],
        [4, 5],
        query([4, 5], [4, 5])
    ));
}

#[test]
fn endpoint_regions_require_points_before_exact_segment_interpretation() {
    let point = query([0, 0], [0, 0]);
    let end = query([10, 10], [10, 10]);
    let probe = query([4, 5], [4, 5]);
    assert_eq!(
        endpoint_segment_intersects_box(point, end, probe),
        Ok(false)
    );
    let contact = query([4, 5], [4, 5]);
    assert_eq!(
        endpoint_segment_intersects_box(point, contact, probe),
        Ok(true)
    );
    let nonpoint = query([0, 0], [1, 1]);
    assert_eq!(
        endpoint_segment_intersects_box(nonpoint, end, probe),
        Err(SegmentBoxDenial::NonPointEndpoint)
    );
    assert_eq!(
        endpoint_segment_intersects_box(point, nonpoint, probe),
        Err(SegmentBoxDenial::NonPointEndpoint)
    );
}
