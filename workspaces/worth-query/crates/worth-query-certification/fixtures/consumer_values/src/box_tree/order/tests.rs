//! Independent explicit interleaving includes all 130 bits, never a packed u128.
use super::*;
use crate::Box2Coordinates;
fn oracle(bounds: Box2, source: usize) -> (Vec<u8>, [i64; 4], usize) {
    let raw = bounds.coordinates();
    let mut bits = Vec::new();
    for bit in (0..65).rev() {
        for axis in 0..2 {
            let center = i128::from(raw.lower[axis]) + i128::from(raw.upper[axis]) + (1_i128 << 64);
            bits.push(((center >> bit) & 1) as u8);
        }
    }
    assert_eq!(bits.len(), 130);
    (
        bits,
        [raw.lower[0], raw.lower[1], raw.upper[0], raw.upper[1]],
        source,
    )
}
fn sample(lower: [i64; 2], upper: [i64; 2]) -> Box2 {
    Box2::new(Box2Coordinates { lower, upper }).unwrap()
}
#[test]
fn exact_centroid_order_keeps_extremes_fractional_centers_and_occurrence_ties() {
    let min = sample([i64::MIN; 2], [i64::MIN; 2]);
    let max = sample([i64::MAX; 2], [i64::MAX; 2]);
    assert_eq!(key(min, 0).0 .0, [0; 2]);
    assert_eq!(key(max, 0).0 .0, [(1_u128 << 65) - 2; 2]);
    let lower_half = sample([-1, 0], [0, 0]);
    let zero = sample([0, 0], [0, 0]);
    assert!(
        key(lower_half, 0) < key(zero, 0),
        "negative half must not round to zero"
    );
    let symmetric = sample([-1, -1], [1, 1]);
    assert_eq!(key(symmetric, 0).0 .0, key(zero, 0).0 .0);
    assert!(
        key(symmetric, 0) < key(zero, 0),
        "full raw bounds break equal centroids"
    );
    assert!(
        key(zero, 7) < key(zero, 9),
        "occurrences remain distinct at identical bounds"
    );
}
#[test]
fn centroid_morton_comparison_agrees_with_full_width_interleaving() {
    let mut samples = vec![
        sample([i64::MIN; 2], [i64::MIN; 2]),
        sample([i64::MAX; 2], [i64::MAX; 2]),
        sample([i64::MIN; 2], [i64::MAX; 2]),
        sample([0; 2], [0; 2]),
        sample([-1; 2], [1; 2]),
        sample([-1, 0], [0, 0]),
        sample([0, -1], [0, 0]),
        sample([i64::MIN, i64::MAX], [i64::MIN, i64::MAX]),
        sample([i64::MAX, i64::MIN], [i64::MAX, i64::MIN]),
    ];
    let mut state = 0x9196_u64;
    for _ in 0..64 {
        let mut next = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            i64::from_ne_bytes(state.to_ne_bytes())
        };
        let (a, b, c, d) = (next(), next(), next(), next());
        samples.push(sample([a.min(b), c.min(d)], [a.max(b), c.max(d)]));
    }
    samples.push(samples[3]);
    for (a, bounds_a) in samples.iter().enumerate() {
        for (b, bounds_b) in samples.iter().enumerate() {
            assert_eq!(
                key(*bounds_a, a).cmp(&key(*bounds_b, b)),
                oracle(*bounds_a, a).cmp(&oracle(*bounds_b, b)),
                "pair {a}/{b}"
            );
        }
    }
    let mut actual = (0..samples.len()).collect::<Vec<_>>();
    actual.sort_by_key(|index| key(samples[*index], *index));
    let mut expected = (0..samples.len()).collect::<Vec<_>>();
    expected.sort_by_key(|index| oracle(samples[*index], *index));
    assert_eq!(actual, expected);
}
