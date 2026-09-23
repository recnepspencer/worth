mod membership;
mod open;
mod published_replacement;
mod world;

use worth_store_lsm_authority::LsmMembershipOwnerCaseObservation;

#[derive(Debug)]
pub struct LsmOwnerCaseObservations {
    membership: Vec<LsmMembershipOwnerCaseObservation>,
}

impl LsmOwnerCaseObservations {
    pub fn membership(&self) -> impl Iterator<Item = LsmMembershipOwnerCaseObservation> + '_ {
        self.membership.iter().copied()
    }
}

pub fn observe_lsm_owner_cases() -> LsmOwnerCaseObservations {
    LsmOwnerCaseObservations {
        membership: open::observe()
            .into_iter()
            .chain(membership::observe())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use worth_store_lsm_authority::{lsm_membership_owner_case_inventory, LsmMembershipOperation};

    use super::observe_lsm_owner_cases;

    #[test]
    fn implemented_membership_operations_equal_their_owner_inventories() {
        let observed = observe_lsm_owner_cases()
            .membership()
            .map(|case| case.id())
            .collect::<BTreeSet<_>>();
        let implemented = [
            LsmMembershipOperation::Open,
            LsmMembershipOperation::PersistRecord,
            LsmMembershipOperation::SelectCompaction,
        ];
        let declared = lsm_membership_owner_case_inventory()
            .filter(|case| implemented.contains(&case.id().operation()))
            .map(|case| case.id())
            .collect::<BTreeSet<_>>();
        assert_eq!(observed, declared);
    }
}
