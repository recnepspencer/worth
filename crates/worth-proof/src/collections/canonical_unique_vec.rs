use crate::proof::{CanonicalOrder, Proof, StructuralProofAuthority, Uniqueness};

/// Strictly increasing keys with checked canonical-order and uniqueness proofs.
#[derive(Debug, PartialEq, Eq)]
pub struct CanonicalUniqueVec<K> {
    keys: Vec<K>,
    order: Proof<CanonicalOrder, StructuralProofAuthority>,
    uniqueness: Proof<Uniqueness, StructuralProofAuthority>,
}

impl<K> CanonicalUniqueVec<K> {
    pub fn try_from_sorted_unique(keys: Vec<K>) -> Result<Self, Vec<K>>
    where
        K: Ord,
    {
        if !keys.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(keys);
        }
        Ok(Self {
            keys,
            order: Proof::mint(),
            uniqueness: Proof::mint(),
        })
    }

    pub fn as_slice(&self) -> &[K] {
        &self.keys
    }

    pub fn order_proof(&self) -> &Proof<CanonicalOrder, StructuralProofAuthority> {
        &self.order
    }

    pub fn uniqueness_proof(&self) -> &Proof<Uniqueness, StructuralProofAuthority> {
        &self.uniqueness
    }

    pub fn into_keys(self) -> Vec<K> {
        self.keys
    }
}

#[cfg(test)]
mod tests {
    use super::CanonicalUniqueVec;

    #[test]
    fn strictly_ordered_keys_admit_empty_and_nonempty_families() {
        assert!(CanonicalUniqueVec::<u64>::try_from_sorted_unique(vec![]).is_ok());
        let keys = CanonicalUniqueVec::try_from_sorted_unique(vec![2, 5, 9]).unwrap();
        assert_eq!(keys.as_slice(), &[2, 5, 9]);
        let _ = keys.order_proof();
        let _ = keys.uniqueness_proof();
    }

    #[test]
    fn duplicates_and_descending_keys_cannot_carry_proofs() {
        assert_eq!(
            CanonicalUniqueVec::try_from_sorted_unique(vec![2, 2]).unwrap_err(),
            vec![2, 2]
        );
        assert_eq!(
            CanonicalUniqueVec::try_from_sorted_unique(vec![3, 1]).unwrap_err(),
            vec![3, 1]
        );
    }
}
