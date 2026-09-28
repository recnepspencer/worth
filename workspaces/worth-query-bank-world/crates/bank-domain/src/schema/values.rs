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

impl PaymentStatus {
    /// Stable text for identity derivation. These bytes are part of every
    /// derived assessment identity, so they never follow a rename.
    pub const fn canonical_text(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::ApprovalRequired => "ApprovalRequired",
            Self::Committed => "Committed",
            Self::Rejected => "Rejected",
            Self::Reversed => "Reversed",
        }
    }
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

    use super::{PaymentStatus, PostingPurpose, PostingPurposeBinding};

    #[test]
    fn payment_status_canonical_text_is_pinned() {
        let pinned = [
            (PaymentStatus::Pending, "Pending"),
            (PaymentStatus::ApprovalRequired, "ApprovalRequired"),
            (PaymentStatus::Committed, "Committed"),
            (PaymentStatus::Rejected, "Rejected"),
            (PaymentStatus::Reversed, "Reversed"),
        ];
        for (status, text) in pinned {
            assert_eq!(status.canonical_text(), text);
        }
    }

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
