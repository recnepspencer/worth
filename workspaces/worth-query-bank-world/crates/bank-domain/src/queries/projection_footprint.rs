//! Projected Bank scalars hold their entire representation inline.
use crate::estate::{
    EmergencyAccessId, EmergencyAccessReason, EmergencyAccessStatus, EstateCaseId, EstateMoment,
    LegalAuthorityId, LegalAuthorityKind, MandatoryReviewId, MandatoryReviewKind,
    MandatoryReviewStatus,
};
use crate::model::{AccountAuthorizationId, AccountId, BankPrincipalId, CustomerRole};
use worth_query_host::facade::historical_cursor::KnownProjectedHeapBytes;

macro_rules! inline_projected_values {
    ($($value:ty),+ $(,)?) => {
        $(
            impl KnownProjectedHeapBytes for $value {
                fn known_heap_bytes(&self) -> usize {
                    0
                }
            }
        )+
    };
}

inline_projected_values!(
    AccountAuthorizationId,
    AccountId,
    BankPrincipalId,
    CustomerRole,
    EmergencyAccessId,
    EmergencyAccessReason,
    EmergencyAccessStatus,
    EstateCaseId,
    EstateMoment,
    LegalAuthorityId,
    LegalAuthorityKind,
    MandatoryReviewId,
    MandatoryReviewKind,
    MandatoryReviewStatus,
);
