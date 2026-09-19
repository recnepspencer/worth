use worth_query_host::facade::primary_graph::{
    WorthQueryOperationProjectionDenial, WorthQueryOperationProjectionDenialKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankEstateOperationProjectionDenial {
    Authorization(crate::BankAuthorizationDenial),
    AuthorizationLineageUnavailable(crate::BankAuthorizationDenial),
    InvariantAdmission(
        worth_query_host::facade::primary_graph::WorthQueryInvariantProjectionDenialKind,
    ),
    WorkBudgetExceeded,
}

pub(super) fn from_query(
    denial: WorthQueryOperationProjectionDenial,
) -> BankEstateOperationProjectionDenial {
    match denial.kind() {
        WorthQueryOperationProjectionDenialKind::Authorization(kind) => {
            match denial.into_authorization_denial() {
                Some(authorization) => BankEstateOperationProjectionDenial::Authorization(
                    crate::BankAuthorizationDenial::from_query(authorization),
                ),
                None => BankEstateOperationProjectionDenial::AuthorizationLineageUnavailable(
                    crate::BankAuthorizationDenial::from_kind(kind, 0),
                ),
            }
        }
        WorthQueryOperationProjectionDenialKind::WorkBudgetExceeded => {
            BankEstateOperationProjectionDenial::WorkBudgetExceeded
        }
        WorthQueryOperationProjectionDenialKind::InvariantAdmission(kind) => {
            BankEstateOperationProjectionDenial::InvariantAdmission(kind)
        }
    }
}
