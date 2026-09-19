use worth_query_decl::facade::{
    application_query::{ApplicationQueryOptionalResultFieldRef, ApplicationQueryResultFieldRef},
    application_schema::{
        DeclaredApplicationUnit, EqualityPredicate, NoApplicationUnit, NoEqualityPredicate,
        ReadOnly, ReadWrite,
    },
};

use crate::{
    estate::{
        BranchId, CapabilityGrantId, CapabilityGrantStatus, DelegationLimit,
        EstateCapabilityOperation, EstateCapabilityPurpose, EstateMoment, EstateWorkflowStage,
        RestrictedBankField,
    },
    model::{AccountId, BankPrincipalId, InstitutionId, Money, USD},
    schema::{
        Account, AccountIdentity, BankSchema, Branch, BranchIdentity, BranchIdentityField,
        CapabilityAmountCeilingField, CapabilityDelegationLimitField, CapabilityDisclosureField,
        CapabilityGrant, CapabilityGrantIdentityField, CapabilityGrantRecord,
        CapabilityGrantStatusField, CapabilityOperationField, CapabilityPurposeField,
        CapabilityValidFromField, CapabilityValidThroughField, CapabilityWorkflowStageField,
        Identity, Institution, InstitutionIdentity, InstitutionIdentityField, Principal,
        PrincipalIdentity, PrincipalIdentityField, UsdCurrency,
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
    CapabilityIdSlot,
    CapabilityOperationSlot,
    CapabilityPurposeSlot,
    CapabilityFieldSlot,
    CapabilityAmountSlot,
    CapabilityValidFromSlot,
    CapabilityValidThroughSlot,
    CapabilityDelegationSlot,
    CapabilityWorkflowSlot,
    CapabilityStatusSlot,
    CapabilityGranteeSlot,
    CapabilityGrantorSlot,
    CapabilityAccountIdentitySlot,
    CapabilityInstitutionIdentitySlot,
    CapabilityBranchIdentitySlot,
    CapabilityParentIdentitySlot,
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
    capability_id,
    CapabilityIdSlot,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityGrantIdentityField,
    CapabilityGrantId,
    ReadOnly,
    "capability"
);
selector!(
    capability_operation,
    CapabilityOperationSlot,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityOperationField,
    EstateCapabilityOperation,
    ReadWrite,
    "operation"
);
selector!(
    capability_purpose,
    CapabilityPurposeSlot,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityPurposeField,
    EstateCapabilityPurpose,
    ReadWrite,
    "purpose"
);
selector!(
    capability_valid_from,
    CapabilityValidFromSlot,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityValidFromField,
    EstateMoment,
    ReadWrite,
    "valid_from"
);
selector!(
    capability_valid_through,
    CapabilityValidThroughSlot,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityValidThroughField,
    EstateMoment,
    ReadWrite,
    "valid_through"
);
selector!(
    capability_delegation,
    CapabilityDelegationSlot,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityDelegationLimitField,
    DelegationLimit,
    ReadWrite,
    "delegation"
);
selector!(
    capability_workflow,
    CapabilityWorkflowSlot,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityWorkflowStageField,
    EstateWorkflowStage,
    ReadWrite,
    "workflow_stage"
);
selector!(
    capability_status,
    CapabilityStatusSlot,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityGrantStatusField,
    CapabilityGrantStatus,
    ReadWrite,
    "status"
);
selector!(
    capability_grantee_identity,
    CapabilityGranteeSlot,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    "grantee"
);
selector!(
    capability_grantor_identity,
    CapabilityGrantorSlot,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    "grantor"
);
selector!(
    capability_account_identity,
    CapabilityAccountIdentitySlot,
    Account,
    Identity,
    AccountIdentity,
    AccountId,
    ReadOnly,
    "account"
);
selector!(
    capability_institution_identity,
    CapabilityInstitutionIdentitySlot,
    Institution,
    InstitutionIdentity,
    InstitutionIdentityField,
    InstitutionId,
    ReadOnly,
    "institution"
);
selector!(
    capability_branch_identity,
    CapabilityBranchIdentitySlot,
    Branch,
    BranchIdentity,
    BranchIdentityField,
    BranchId,
    ReadOnly,
    "branch"
);
selector!(
    capability_parent_identity,
    CapabilityParentIdentitySlot,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityGrantIdentityField,
    CapabilityGrantId,
    ReadOnly,
    "parent"
);

pub(in crate::queries::estate) fn capability_field() -> ApplicationQueryOptionalResultFieldRef<
    EstateGovernanceQuery,
    CapabilityFieldSlot,
    BankSchema,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityDisclosureField,
    RestrictedBankField,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryOptionalResultFieldRef::new("field", CapabilityDisclosureField::reference())
}

pub(in crate::queries::estate) fn capability_amount() -> ApplicationQueryOptionalResultFieldRef<
    EstateGovernanceQuery,
    CapabilityAmountSlot,
    BankSchema,
    CapabilityGrant,
    CapabilityGrantRecord,
    CapabilityAmountCeilingField,
    Money<USD>,
    ReadWrite,
    NoEqualityPredicate,
    DeclaredApplicationUnit<UsdCurrency, USD>,
> {
    ApplicationQueryOptionalResultFieldRef::new(
        "amount_ceiling",
        CapabilityAmountCeilingField::reference(),
    )
}
