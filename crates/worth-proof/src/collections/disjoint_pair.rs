use crate::proof::{Disjointness, Proof, StructuralProofAuthority};

use super::Pair;

#[derive(Debug, PartialEq, Eq)]
/// A checked pair whose two values are unequal.
///
/// Public construction is intentionally limited to [`Self::try_from_disjoint`].
/// The returned value carries the structural proof established by that check;
/// accessors inspect it, and [`Self::into_parts`] consumes it without opening a
/// second proof-minting lane.
pub struct DisjointPair<T> {
    pair: Pair<T>,
    proof: Proof<Disjointness, StructuralProofAuthority>,
}

impl<T> DisjointPair<T> {
    pub(crate) fn new(
        left: T,
        right: T,
        proof: Proof<Disjointness, StructuralProofAuthority>,
    ) -> Self {
        Self {
            pair: Pair::new(left, right),
            proof,
        }
    }

    /// Admit two values as disjoint, minting the proof on success.
    ///
    /// The checked door, matching [`super::CanonicalVec::try_from_sorted`] and
    /// [`super::UniqueVec::try_from_unique`]: a caller supplies raw values, this
    /// establishes the fact, and only then does the proof exist. There is no
    /// path that produces a `DisjointPair` without disjointness having been
    /// checked here.
    pub fn try_from_disjoint(left: T, right: T) -> Result<Self, Pair<T>>
    where
        T: PartialEq,
    {
        if left == right {
            return Err(Pair::new(left, right));
        }
        Ok(Self::new(
            left,
            right,
            Proof::<Disjointness, StructuralProofAuthority>::mint(),
        ))
    }

    pub fn left(&self) -> &T {
        self.pair.left()
    }

    pub fn right(&self) -> &T {
        self.pair.right()
    }

    pub fn pair(&self) -> &Pair<T> {
        &self.pair
    }

    pub fn proof(&self) -> &Proof<Disjointness, StructuralProofAuthority> {
        &self.proof
    }

    pub fn into_parts(self) -> (Pair<T>, Proof<Disjointness, StructuralProofAuthority>) {
        (self.pair, self.proof)
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::DisjointPair;
    use crate::collections::Pair;

    const _: () = assert!(size_of::<DisjointPair<u64>>() == size_of::<Pair<u64>>());

    #[test]
    fn checked_public_introduction_accepts_only_disjoint_values() {
        let admitted =
            DisjointPair::try_from_disjoint("left", "right").expect("unequal values are disjoint");
        assert_eq!(admitted.left(), &"left");
        assert_eq!(admitted.right(), &"right");

        let rejected = DisjointPair::try_from_disjoint("same", "same")
            .expect_err("equal values are not disjoint");
        assert_eq!(rejected.into_array(), ["same", "same"]);
    }
}
