// A sibling of a `limit_authority!` owner cannot call the owner's `refuse`
// door, so it cannot mint a limit of the owner's dimension.
use worth_foundational::{ExhaustedLimit, LimitCounts};

mod owner {
    worth_foundational::limit_authority!(pub OwnerAuthority);

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
        super::owner::OwnerAuthority::refuse(
            super::owner::OwnerBound::Entries,
            LimitCounts::new(1, 0),
        )
    }
}

fn main() {
    let _ = sibling::forge();
}
