use worth_query_decl::facade::{
    application_query::ApplicationQueryResultFieldRef,
    application_schema::{EqualityPredicate, NoApplicationUnit, ReadOnly, ReadWrite},
};

use crate::{
    estate::{EmergencyAccessId, EmergencyAccessReason, EmergencyAccessStatus, EstateMoment},
    model::BankPrincipalId,
    schema::{
        BankSchema, EmergencyAccess, EmergencyAccessExpiresAtField, EmergencyAccessIdentityField,
        EmergencyAccessIssuedAtField, EmergencyAccessReasonField, EmergencyAccessRecord,
        EmergencyAccessStatusField, Principal, PrincipalIdentity, PrincipalIdentityField,
    },
};

use super::super::governance::EstateGovernanceQuery;

macro_rules! slots {
    ($($name:ident),+ $(,)?) => {
        $(
            pub(in crate::queries::estate) struct $name;
            impl worth_query_decl::facade::portable_identity::WorthQueryPortableType for $name {
                const PORTABLE_TYPE_NAME: &'static str = stringify!($name);
            }
        )+
    };
}

slots!(
    EmergencyIdSlot,
    EmergencyReasonSlot,
    EmergencyStatusSlot,
    EmergencyIssuedAtSlot,
    EmergencyExpiresAtSlot,
    EmergencyRequesterSlot,
    EmergencyApproverSlot,
);

macro_rules! selector {
    ($name:ident, $slot:ty, $entity:ty, $aspect:ty, $field:ty, $value:ty, $write:ty, $alias:literal) => {
        pub(in crate::queries::estate) fn $name() -> ApplicationQueryResultFieldRef<
            EstateGovernanceQuery,
            $slot,
            BankSchema,
            $entity,
            $aspect,
            $field,
            $value,
            $write,
            EqualityPredicate,
            NoApplicationUnit,
        > {
            ApplicationQueryResultFieldRef::new($alias, <$field>::reference())
        }
    };
}

selector!(
    emergency_id,
    EmergencyIdSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessIdentityField,
    EmergencyAccessId,
    ReadOnly,
    "emergency"
);
selector!(
    emergency_reason,
    EmergencyReasonSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessReasonField,
    EmergencyAccessReason,
    ReadWrite,
    "reason"
);
selector!(
    emergency_status,
    EmergencyStatusSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessStatusField,
    EmergencyAccessStatus,
    ReadWrite,
    "status"
);
selector!(
    emergency_issued_at,
    EmergencyIssuedAtSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessIssuedAtField,
    EstateMoment,
    ReadWrite,
    "issued_at"
);
selector!(
    emergency_expires_at,
    EmergencyExpiresAtSlot,
    EmergencyAccess,
    EmergencyAccessRecord,
    EmergencyAccessExpiresAtField,
    EstateMoment,
    ReadWrite,
    "expires_at"
);
selector!(
    emergency_requester_identity,
    EmergencyRequesterSlot,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    "requester"
);
selector!(
    emergency_approver_identity,
    EmergencyApproverSlot,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    "approver"
);
