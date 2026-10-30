pub use bank_domain::{
    estate::{
        DeathNoticeStatus, EstateCaseStatus, EstateWorkflowStage, LegalAuthorityKind,
        MandatoryReviewKind, MandatoryReviewStatus,
    },
    model::EmployeeRole,
};
pub use bank_server::{
    queries, BankApplicationQueryDenial, BankAuthorizationDenialKind,
    BankProductSelectionDenialKind, BankReadControls,
};

pub use super::estate_fixture::estate_read_world;
pub use crate::facade::support::request_scope;

pub fn controls() -> BankReadControls {
    BankReadControls::current(request_scope(), 4).unwrap()
}
