//! How far, and which ways, a drag moved the window, in logical points.

/// The width the proof must cross both ways: Platform Pulse's layout
/// breakpoint, `width from 1200`. The interval is half-open, so a viewport
/// 1200 points or wider is wide and one narrower is stacked.
pub const BREAKPOINT_WIDTH: u32 = 1200;
/// The milestone's journey runs from at least this extent...
pub const LARGE_EXTENT: [u32; 2] = [1536, 1024];
/// ...down to at most this one, and back.
pub const SMALL_EXTENT: [u32; 2] = [800, 600];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Coverage {
    /// Rounded to whole points for display.
    pub smallest: [u32; 2],
    pub largest: [u32; 2],
    /// Whether some extent was at most [`SMALL_EXTENT`], and some at least
    /// [`LARGE_EXTENT`], compared exactly.
    pub reached_small: bool,
    pub reached_large: bool,
    /// Changes of direction in width plus height.
    pub reversals: usize,
    /// Steps from at least [`BREAKPOINT_WIDTH`] to below it.
    pub narrowing: usize,
    /// Steps from below [`BREAKPOINT_WIDTH`] to at least it.
    pub widening: usize,
}

/// A physical extent in logical points at `dpi`, 96 being one pixel per point.
pub fn logical(extent: [u32; 2], dpi: u32) -> [u32; 2] {
    let dpi = if dpi == 0 { 96 } else { dpi };
    extent.map(|side| ((u64::from(side) * 96 + u64::from(dpi / 2)) / u64::from(dpi)) as u32)
}

/// A physical side and a side in points at `dpi`, scaled to compare exactly
/// rather than after rounding to whole points.
fn scaled(side: u32, points: u32, dpi: u32) -> (u64, u64) {
    let dpi = if dpi == 0 { 96 } else { dpi };
    (u64::from(side) * 96, u64::from(points) * u64::from(dpi))
}

fn wide(extent: [u32; 2], dpi: u32) -> bool {
    let (side, breakpoint) = scaled(extent[0], BREAKPOINT_WIDTH, dpi);
    side >= breakpoint
}

fn at_most(extent: [u32; 2], points: [u32; 2], dpi: u32) -> bool {
    (0..2).all(|axis| {
        let (side, limit) = scaled(extent[axis], points[axis], dpi);
        side <= limit
    })
}

fn at_least(extent: [u32; 2], points: [u32; 2], dpi: u32) -> bool {
    (0..2).all(|axis| {
        let (side, limit) = scaled(extent[axis], points[axis], dpi);
        side >= limit
    })
}

fn reversals(extents: &[[u32; 2]]) -> usize {
    let mut direction = 0_i64;
    let mut count = 0;
    for pair in extents.windows(2) {
        let step = i64::from(pair[1][0] + pair[1][1]) - i64::from(pair[0][0] + pair[0][1]);
        let sign = step.signum();
        if sign != 0 {
            if direction != 0 && sign != direction {
                count += 1;
            }
            direction = sign;
        }
    }
    count
}

/// The coverage of physical `extents`, in the order the host consumed them.
pub fn coverage(extents: &[[u32; 2]], dpi: u32) -> Coverage {
    let points: Vec<[u32; 2]> = extents.iter().map(|&extent| logical(extent, dpi)).collect();
    let Some(&first) = points.first() else {
        return Coverage::default();
    };
    let mut coverage = Coverage {
        smallest: first,
        largest: first,
        reached_small: extents
            .iter()
            .any(|&extent| at_most(extent, SMALL_EXTENT, dpi)),
        reached_large: extents
            .iter()
            .any(|&extent| at_least(extent, LARGE_EXTENT, dpi)),
        reversals: reversals(&points),
        ..Coverage::default()
    };
    for pair in extents.windows(2) {
        let [before, after] = [wide(pair[0], dpi), wide(pair[1], dpi)];
        coverage.narrowing += usize::from(before && !after);
        coverage.widening += usize::from(!before && after);
    }
    for extent in &points {
        coverage.smallest = [
            coverage.smallest[0].min(extent[0]),
            coverage.smallest[1].min(extent[1]),
        ];
        coverage.largest = [
            coverage.largest[0].max(extent[0]),
            coverage.largest[1].max(extent[1]),
        ];
    }
    coverage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_extents_convert_to_logical_points() {
        assert_eq!(logical([2304, 1536], 144), [1536, 1024]);
        assert_eq!(logical([1200, 900], 144), [800, 600]);
        assert_eq!(logical([800, 600], 96), [800, 600]);
        assert_eq!(logical([800, 600], 0), [800, 600]);
    }

    #[test]
    fn the_breakpoint_counts_only_crossings_in_each_direction() {
        let down_and_back = [
            [1536, 1024],
            [1200, 800],
            [1199, 800],
            [800, 600],
            [1200, 800],
            [1536, 1024],
        ];
        let coverage = coverage(&down_and_back, 96);
        assert_eq!((coverage.narrowing, coverage.widening), (1, 1));
        assert_eq!(coverage.smallest, [800, 600]);
        assert_eq!(coverage.largest, [1536, 1024]);
        assert!(coverage.reached_small && coverage.reached_large);
        assert_eq!(coverage.reversals, 1);

        let up_then_jiggle = [
            [800, 600],
            [1536, 1024],
            [1500, 1000],
            [1536, 1024],
            [1500, 1000],
        ];
        let coverage = super::coverage(&up_then_jiggle, 96);
        assert_eq!((coverage.narrowing, coverage.widening), (0, 1));
        assert_eq!(coverage.reversals, 3);

        let starting_between_the_milestone_width_and_the_breakpoint =
            [[1150, 800], [800, 600], [1536, 1024], [1500, 1000]];
        let coverage =
            super::coverage(&starting_between_the_milestone_width_and_the_breakpoint, 96);
        assert_eq!((coverage.narrowing, coverage.widening), (0, 1));

        let at_192_dpi = [[2400, 1600], [2399, 1600], [2400, 1600]];
        let coverage = super::coverage(&at_192_dpi, 192);
        assert_eq!(
            (coverage.narrowing, coverage.widening),
            (1, 1),
            "2399 pixels is 1199.5 points, narrower than the breakpoint though it rounds to 1200"
        );

        let just_short = super::coverage(&[[1601, 1201], [3071, 2047]], 192);
        assert_eq!(
            (just_short.smallest, just_short.largest),
            ([801, 601], [1536, 1024]),
            "rounded for display"
        );
        assert!(!just_short.reached_small && !just_short.reached_large);

        assert_eq!(super::coverage(&[], 96), Coverage::default());
    }
}
