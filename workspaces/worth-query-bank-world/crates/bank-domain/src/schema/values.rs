mod bindings;

pub use bindings::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountKind {
    Personal,
    Business,
    InstitutionCash,
    InstitutionSettlement,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountStatus {
    Open,
    Frozen,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    ApprovalRequired,
    Committed,
    Rejected,
    Reversed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostingPurpose {
    OpeningFunding,
    Deposit,
    Withdrawal,
    Transfer,
    EstateDisbursement,
    Reversal,
}

#[cfg(test)]
mod tests {
    use worth_query_decl::facade::application_schema::{
        ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    };

    use super::{PostingPurpose, PostingPurposeBinding};

    #[test]
    fn estate_disbursement_is_a_distinct_round_tripping_posting_purpose() {
        let encoded = PostingPurposeBinding::encode(&PostingPurpose::EstateDisbursement).unwrap();

        assert_eq!(
            PostingPurposeBinding::decode(&encoded).unwrap(),
            PostingPurpose::EstateDisbursement
        );
        assert_ne!(
            encoded,
            PostingPurposeBinding::encode(&PostingPurpose::Transfer).unwrap(),
            "estate disbursement must not masquerade as an ordinary transfer"
        );
    }
}
