use std::collections::BTreeMap;

use crate::proof::{CanonicalOrder, Disjointness, Proof, StructuralProofAuthority, Uniqueness};

/// A family ordered by opaque member identity, with strictly ordered keys in
/// each member and no key shared by two members.
#[derive(Debug, PartialEq, Eq)]
pub struct DisjointKeySetFamily<Id, K> {
    members: Vec<(Id, Vec<K>)>,
    member_order: Proof<CanonicalOrder, StructuralProofAuthority>,
    uniqueness: Proof<Uniqueness, StructuralProofAuthority>,
    disjointness: Proof<Disjointness, StructuralProofAuthority>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisjointKeySetViolation {
    MemberIdentityOrder {
        earlier: usize,
        later: usize,
    },
    KeyOrder {
        member: usize,
        earlier: usize,
        later: usize,
    },
    IntersectingKey {
        earlier_member: usize,
        later_member: usize,
        later_key: usize,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct DisjointKeySetDenial<Id, K> {
    members: Vec<(Id, Vec<K>)>,
    violation: DisjointKeySetViolation,
}

impl<Id, K> DisjointKeySetDenial<Id, K> {
    pub fn violation(&self) -> DisjointKeySetViolation {
        self.violation
    }

    pub fn members(&self) -> &[(Id, Vec<K>)] {
        &self.members
    }

    pub fn into_members(self) -> Vec<(Id, Vec<K>)> {
        self.members
    }
}

impl<Id, K> DisjointKeySetFamily<Id, K> {
    pub fn try_from_sorted_sets(
        members: Vec<(Id, Vec<K>)>,
    ) -> Result<Self, DisjointKeySetDenial<Id, K>>
    where
        Id: Ord,
        K: Ord,
    {
        if let Some(violation) = first_violation(&members) {
            return Err(DisjointKeySetDenial { members, violation });
        }
        Ok(Self {
            members,
            member_order: Proof::mint(),
            uniqueness: Proof::mint(),
            disjointness: Proof::mint(),
        })
    }

    pub fn members(&self) -> &[(Id, Vec<K>)] {
        &self.members
    }

    pub fn member_order_proof(&self) -> &Proof<CanonicalOrder, StructuralProofAuthority> {
        &self.member_order
    }

    pub fn uniqueness_proof(&self) -> &Proof<Uniqueness, StructuralProofAuthority> {
        &self.uniqueness
    }

    pub fn disjointness_proof(&self) -> &Proof<Disjointness, StructuralProofAuthority> {
        &self.disjointness
    }

    pub fn into_members(self) -> Vec<(Id, Vec<K>)> {
        self.members
    }
}

fn first_violation<Id: Ord, K: Ord>(members: &[(Id, Vec<K>)]) -> Option<DisjointKeySetViolation> {
    let mut seen: BTreeMap<&K, usize> = BTreeMap::new();
    for (member_index, (identity, keys)) in members.iter().enumerate() {
        if member_index > 0 && members[member_index - 1].0 >= *identity {
            return Some(DisjointKeySetViolation::MemberIdentityOrder {
                earlier: member_index - 1,
                later: member_index,
            });
        }
        for (key_index, key) in keys.iter().enumerate() {
            if key_index > 0 && keys[key_index - 1] >= *key {
                return Some(DisjointKeySetViolation::KeyOrder {
                    member: member_index,
                    earlier: key_index - 1,
                    later: key_index,
                });
            }
            if let Some(&earlier_member) = seen.get(key) {
                return Some(DisjointKeySetViolation::IntersectingKey {
                    earlier_member,
                    later_member: member_index,
                    later_key: key_index,
                });
            }
            seen.insert(key, member_index);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{DisjointKeySetFamily, DisjointKeySetViolation};

    #[test]
    fn disjoint_sets_keep_their_order_and_proofs() {
        let family = DisjointKeySetFamily::try_from_sorted_sets(vec![
            (1_u8, vec![2_u8, 4]),
            (3, vec![]),
            (7, vec![1, 3]),
        ])
        .unwrap();
        assert_eq!(family.members().len(), 3);
        let _ = family.member_order_proof();
        let _ = family.uniqueness_proof();
        let _ = family.disjointness_proof();
    }

    #[test]
    fn overlapping_sets_name_both_members_and_preserve_input() {
        let input = vec![(1_u8, vec![2_u8, 4]), (3, vec![1, 4])];
        let denial = DisjointKeySetFamily::try_from_sorted_sets(input.clone()).unwrap_err();
        assert_eq!(denial.members(), input);
        assert_eq!(
            denial.violation(),
            DisjointKeySetViolation::IntersectingKey {
                earlier_member: 0,
                later_member: 1,
                later_key: 1,
            }
        );
    }

    #[test]
    fn unordered_or_duplicate_identities_and_keys_are_denied() {
        let cases = [
            (
                vec![(2, vec![]), (1, vec![])],
                DisjointKeySetViolation::MemberIdentityOrder {
                    earlier: 0,
                    later: 1,
                },
            ),
            (
                vec![(1, vec![]), (1, vec![])],
                DisjointKeySetViolation::MemberIdentityOrder {
                    earlier: 0,
                    later: 1,
                },
            ),
            (
                vec![(1, vec![3, 2])],
                DisjointKeySetViolation::KeyOrder {
                    member: 0,
                    earlier: 0,
                    later: 1,
                },
            ),
            (
                vec![(1, vec![2, 2])],
                DisjointKeySetViolation::KeyOrder {
                    member: 0,
                    earlier: 0,
                    later: 1,
                },
            ),
        ];
        for (members, expected) in cases {
            assert_eq!(
                DisjointKeySetFamily::try_from_sorted_sets(members)
                    .unwrap_err()
                    .violation(),
                expected
            );
        }
    }
}
