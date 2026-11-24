//! Sealed coordinate ordering, half-open/closed range inclusion, and fixed-size
//! `Copy` AABB envelopes for stock checked ordered managed-index adapters.
use std::cmp::Ordering;
use worth_query_declaration::facade::application_schema::{
    CanonicalDate, CanonicalF64, CanonicalTimestamp,
};
use worth_query_execution::facade::primary_graph::WorthQueryManagedIndexPredicateSummary;

mod private {
    pub trait Sealed {}
}

/// Range and bounding-box boundary inclusion semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRangeInclusion {
    /// Half-open `[min, max)`: valid iff `min < max`; overlaps iff `a.min < b.max && b.min < a.max`.
    HalfOpen,
    /// Closed `[min, max]`: valid iff `min <= max`; overlaps iff `a.min <= b.max && b.min <= a.max`.
    Closed,
}

impl WorthQueryRangeInclusion {
    /// Returns `true` iff `[lo, hi)` (HalfOpen) or `[lo, hi]` (Closed) is valid and non-empty.
    pub fn is_valid_span<C: WorthQueryOrderedCoord>(self, lo: C, hi: C) -> bool {
        if !lo.is_valid_coord() || !hi.is_valid_coord() {
            return false;
        }
        match self {
            Self::HalfOpen => lo.coord_cmp(hi) == Ordering::Less,
            Self::Closed => lo.coord_cmp(hi) != Ordering::Greater,
        }
    }

    /// Returns `true` iff both spans are valid non-empty and overlap under `self`.
    pub fn spans_overlap<C: WorthQueryOrderedCoord>(
        self,
        a_lo: C,
        a_hi: C,
        b_lo: C,
        b_hi: C,
    ) -> bool {
        if !self.is_valid_span(a_lo, a_hi) || !self.is_valid_span(b_lo, b_hi) {
            return false;
        }
        match self {
            Self::HalfOpen => {
                a_lo.coord_cmp(b_hi) == Ordering::Less && b_lo.coord_cmp(a_hi) == Ordering::Less
            }
            Self::Closed => {
                a_lo.coord_cmp(b_hi) != Ordering::Greater
                    && b_lo.coord_cmp(a_hi) != Ordering::Greater
            }
        }
    }
}

/// Typed domain denial returned by stock ordered managed-index adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryStockIndexDenial {
    /// Probe carried empty, reversed, or NaN bounds for the configured inclusion mode.
    InvalidProbeBounds,
}

impl std::fmt::Display for WorthQueryStockIndexDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProbeBounds => {
                f.write_str("managed-index probe bounds are empty or invalid")
            }
        }
    }
}

impl std::error::Error for WorthQueryStockIndexDenial {}

/// Sealed canonical coordinate types supported by stock interval and box indexes.
///
/// For [`CanonicalF64`], comparison uses exact mathematical floating-point order
/// (canonicalizing `-0.0` to `+0.0` for associative envelope `Eq`, and rejecting
/// `NaN` endpoints) rather than raw IEEE-754 `u64` bit ordering.
pub trait WorthQueryOrderedCoord: Copy + Eq + Send + Sync + private::Sealed + 'static {
    fn is_valid_coord(self) -> bool;
    fn normalize_endpoint(self) -> Self;
    fn coord_cmp(self, other: Self) -> Ordering;

    fn min_coord(self, other: Self) -> Self {
        let (a, b) = (self.normalize_endpoint(), other.normalize_endpoint());
        if a.coord_cmp(b) == Ordering::Greater {
            b
        } else {
            a
        }
    }

    fn max_coord(self, other: Self) -> Self {
        let (a, b) = (self.normalize_endpoint(), other.normalize_endpoint());
        if a.coord_cmp(b) == Ordering::Less {
            b
        } else {
            a
        }
    }
}

macro_rules! impl_ord_coord {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl private::Sealed for $ty {}
            impl WorthQueryOrderedCoord for $ty {
                #[inline]
                fn is_valid_coord(self) -> bool {
                    true
                }
                #[inline]
                fn normalize_endpoint(self) -> Self {
                    self
                }
                #[inline]
                fn coord_cmp(self, other: Self) -> Ordering {
                    self.cmp(&other)
                }
            }
        )+
    };
}

impl_ord_coord!(i64, u64, CanonicalDate, CanonicalTimestamp);

impl private::Sealed for CanonicalF64 {}
impl WorthQueryOrderedCoord for CanonicalF64 {
    #[inline]
    fn is_valid_coord(self) -> bool {
        !self.as_f64().is_nan()
    }

    #[inline]
    fn normalize_endpoint(self) -> Self {
        if self.as_f64() == 0.0 {
            Self::from_f64(0.0)
        } else {
            self
        }
    }

    #[inline]
    fn coord_cmp(self, other: Self) -> Ordering {
        let (a, b) = (self.as_f64(), other.as_f64());
        match (a.is_nan(), b.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => {
                let a = if a == 0.0 { 0.0 } else { a };
                let b = if b == 0.0 { 0.0 } else { b };
                a.total_cmp(&b)
            }
        }
    }
}

/// Fixed-size `Copy` associative N-dimensional AABB envelope (`D` in `{1, 2, 3}`).
///
/// Invalid, empty, or reversed source rows contribute [`Self::Empty`], which is
/// the associative identity of [`Self::combine`] and never matches any probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAxisEnvelope<C: WorthQueryOrderedCoord, const D: usize> {
    Empty,
    NonEmpty { min: [C; D], max: [C; D] },
}

impl<C: WorthQueryOrderedCoord, const D: usize> WorthQueryAxisEnvelope<C, D> {
    pub fn from_bounds(min: [C; D], max: [C; D], inclusion: WorthQueryRangeInclusion) -> Self {
        let mut norm_min = min;
        let mut norm_max = max;
        for i in 0..D {
            if !inclusion.is_valid_span(min[i], max[i]) {
                return Self::Empty;
            }
            norm_min[i] = min[i].normalize_endpoint();
            norm_max[i] = max[i].normalize_endpoint();
        }
        Self::NonEmpty {
            min: norm_min,
            max: norm_max,
        }
    }

    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, x) | (x, Self::Empty) => x,
            (
                Self::NonEmpty {
                    min: lmin,
                    max: lmax,
                },
                Self::NonEmpty {
                    min: rmin,
                    max: rmax,
                },
            ) => {
                let mut min = lmin;
                let mut max = lmax;
                for i in 0..D {
                    min[i] = lmin[i].min_coord(rmin[i]);
                    max[i] = lmax[i].max_coord(rmax[i]);
                }
                Self::NonEmpty { min, max }
            }
        }
    }

    pub fn may_overlap_bounds(
        self,
        min: [C; D],
        max: [C; D],
        inclusion: WorthQueryRangeInclusion,
    ) -> bool {
        let Self::NonEmpty {
            min: smin,
            max: smax,
        } = self
        else {
            return false;
        };
        (0..D).all(|i| inclusion.spans_overlap(smin[i], smax[i], min[i], max[i]))
    }

    pub fn lex_cmp(a_min: [C; D], a_max: [C; D], b_min: [C; D], b_max: [C; D]) -> Ordering {
        for i in 0..D {
            let ord = a_min[i].coord_cmp(b_min[i]);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        for i in 0..D {
            let ord = a_max[i].coord_cmp(b_max[i]);
            if ord != Ordering::Equal {
                return ord;
            }
        }
        Ordering::Equal
    }

    pub fn bounds_valid(min: [C; D], max: [C; D], inclusion: WorthQueryRangeInclusion) -> bool {
        (0..D).all(|i| inclusion.is_valid_span(min[i], max[i]))
    }

    pub fn bounds_overlap(
        a_min: [C; D],
        a_max: [C; D],
        b_min: [C; D],
        b_max: [C; D],
        inclusion: WorthQueryRangeInclusion,
    ) -> bool {
        (0..D).all(|i| inclusion.spans_overlap(a_min[i], a_max[i], b_min[i], b_max[i]))
    }
}

impl<C: WorthQueryOrderedCoord, const D: usize> WorthQueryManagedIndexPredicateSummary
    for WorthQueryAxisEnvelope<C, D>
{
    fn merge(own: &Self, children: [Option<&Self>; 2]) -> Self {
        children
            .into_iter()
            .flatten()
            .fold(*own, |acc, child| acc.combine(*child))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_f64_ordering_handles_negatives_signed_zero_and_nan() {
        let neg2 = CanonicalF64::from_f64(-2.5);
        let neg1 = CanonicalF64::from_f64(-1.0);
        let nzero = CanonicalF64::from_f64(-0.0);
        let pzero = CanonicalF64::from_f64(0.0);
        let pos1 = CanonicalF64::from_f64(1.0);
        let nan = CanonicalF64::from_f64(f64::NAN);

        assert_eq!(neg2.coord_cmp(neg1), Ordering::Less);
        assert_eq!(neg1.coord_cmp(pzero), Ordering::Less);
        assert_eq!(nzero.coord_cmp(pzero), Ordering::Equal);
        assert_eq!(pzero.coord_cmp(pos1), Ordering::Less);
        assert_eq!(nzero.min_coord(pzero), pzero);
        assert_eq!(pzero.min_coord(nzero), pzero);
        assert!(!nan.is_valid_coord());
        assert!(!WorthQueryRangeInclusion::Closed.is_valid_span(neg1, nan));
        assert!(!WorthQueryRangeInclusion::HalfOpen.is_valid_span(nan, pos1));
    }

    #[test]
    fn half_open_and_closed_reject_empty_or_reversed_spans_and_preserve_associativity() {
        let ho = WorthQueryRangeInclusion::HalfOpen;
        let cl = WorthQueryRangeInclusion::Closed;

        // Degenerate [10, 10) must NOT match [5, 15) under HalfOpen, but [10, 10] MUST under Closed.
        assert!(!ho.spans_overlap(10i64, 10, 5, 15));
        assert!(!ho.spans_overlap(15i64, 10, 5, 20));
        assert!(cl.spans_overlap(10i64, 10, 5, 15));
        assert!(!cl.spans_overlap(15i64, 10, 5, 20));
        // Edge-touching [0, 10) vs [10, 20): disjoint under HalfOpen, overlapping under Closed.
        assert!(!ho.spans_overlap(0i64, 10, 10, 20));
        assert!(cl.spans_overlap(0i64, 10, 10, 20));

        let empty = WorthQueryAxisEnvelope::from_bounds([10i64, 0], [10, 5], ho);
        assert_eq!(empty, WorthQueryAxisEnvelope::Empty);
        let a = WorthQueryAxisEnvelope::from_bounds([-5i64, 2], [3, 8], ho);
        let b = WorthQueryAxisEnvelope::from_bounds([0i64, -4], [10, 1], ho);
        assert_eq!(empty.combine(a), a);
        assert_eq!(a.combine(empty), a);
        assert_eq!(a.combine(b), b.combine(a));
    }
}
