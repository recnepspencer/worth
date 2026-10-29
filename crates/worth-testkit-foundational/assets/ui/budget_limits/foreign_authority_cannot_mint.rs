// A module with its own sealed authority cannot mint another owner's limit:
// the dimension names its owner's authority, and only that owner's witness
// records the refusal the constructor consumes.
use worth_foundational::{BudgetRefused, ExhaustedLimit, LimitCounts};
use worth_proof::Performed;

mod owner {
    worth_proof::authority_marker!(pub OwnerAuthority);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum OwnerBound {
        Entries,
    }

    impl worth_foundational::LimitDimension for OwnerBound {
        type Authority = OwnerAuthority;
    }
}

mod forger {
    worth_proof::authority_marker!(pub ForgerAuthority);

    pub fn witness() -> worth_proof::AuthorityWitness<ForgerAuthority> {
        ForgerAuthority::witness()
    }
}

fn main() {
    let refusal: Performed<BudgetRefused, forger::ForgerAuthority, LimitCounts> =
        Performed::record(&forger::witness(), LimitCounts::new(1, 0));
    let _limit = ExhaustedLimit::refused(owner::OwnerBound::Entries, refusal);
}
