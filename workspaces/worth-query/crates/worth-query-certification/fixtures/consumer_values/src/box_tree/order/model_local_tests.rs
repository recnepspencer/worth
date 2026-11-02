//! Independent unsigned endpoint sums and explicit 130-bit interleaving.
use super::*;
use crate::{Box2Coordinates, ModelLocalMetre};
type Bounds = IntegerBox2<ModelLocalMetre>;
fn oracle(bounds: Bounds, source: usize) -> (Vec<bool>, [u64; 4], usize) {
    let raw = bounds.coordinates();
    let centers = [
        u128::from(raw.lower[0]) + u128::from(raw.upper[0]),
        u128::from(raw.lower[1]) + u128::from(raw.upper[1]),
    ];
    let mut bits = Vec::new();
    for bit in (0..65).rev() {
        for center in centers {
            bits.push((center & (1 << bit)) != 0);
        }
    }
    (
        bits,
        [raw.lower[0], raw.lower[1], raw.upper[0], raw.upper[1]],
        source,
    )
}
#[test]
fn unsigned_centroid_order_matches_full_width_interleaving_without_rounding() {
    let m = u64::MAX;
    let mut raw = vec![
        [0, 0, 0, 0],
        [m, m, m, m],
        [0, 0, m, m],
        [m, 0, m, 0],
        [0, m, 0, m],
        [m - 1, m - 1, m, m],
        [0, 0, 1, 0],
        [0, 0, 0, 1],
        [0, 0, 0, 0],
    ];
    let mut state = 0x919c_u64;
    for _ in 0..64 {
        let mut next = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            state
        };
        let (a, b, c, d) = (next(), next(), next(), next());
        raw.push([a.min(b), c.min(d), a.max(b), c.max(d)]);
    }
    let boxes: Vec<_> = raw
        .into_iter()
        .map(|[a, b, c, d]| {
            Bounds::new(Box2Coordinates {
                lower: [a, b],
                upper: [c, d],
            })
            .unwrap()
        })
        .collect();
    assert_eq!(key(boxes[0], 0).0 .0, [0; 2]);
    assert_eq!(key(boxes[1], 0).0 .0, [(1_u128 << 65) - 2; 2]);
    for (a, first) in boxes.iter().enumerate() {
        for (b, second) in boxes.iter().enumerate() {
            assert_eq!(
                key(*first, a).cmp(&key(*second, b)),
                oracle(*first, a).cmp(&oracle(*second, b))
            );
        }
    }
    assert_eq!(size_of::<Bounds>(), size_of::<Box2Coordinates<u64>>());
    assert_eq!(size_of::<Bounds>(), size_of::<Box2>());
}
