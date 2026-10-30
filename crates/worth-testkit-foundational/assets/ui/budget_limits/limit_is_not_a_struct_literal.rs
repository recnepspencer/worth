// An exhausted limit and its counts have private fields: a site cannot build
// one from numbers no budget produced.
use worth_foundational::{ExhaustedLimit, LimitCounts};

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

fn main() {
    let _counts = LimitCounts {
        observed: 1,
        admitted: 0,
    };
    let _limit: ExhaustedLimit<owner::OwnerBound> = ExhaustedLimit {
        dimension: owner::OwnerBound::Entries,
        counts: LimitCounts::new(1, 0),
    };
}
