// A sibling of the owning budget module cannot reach the owner's witness, so
// it cannot record a refusal under the owner's authority.
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

mod sibling {
    use super::*;

    pub fn forge() -> ExhaustedLimit<super::owner::OwnerBound> {
        let refusal: Performed<BudgetRefused, _, _> = Performed::record(
            &super::owner::OwnerAuthority::witness(),
            LimitCounts::new(1, 0),
        );
        ExhaustedLimit::refused(super::owner::OwnerBound::Entries, refusal)
    }
}

fn main() {
    let _ = sibling::forge();
}
