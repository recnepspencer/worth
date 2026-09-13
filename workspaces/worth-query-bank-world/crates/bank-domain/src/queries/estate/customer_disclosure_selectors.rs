use worth_query_decl::facade::application_query::{
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef, ExactlyOneResult,
    ForwardResultTraversal, ManyResults, ReverseResultTraversal,
};
use worth_query_decl::facade::application_schema::{
    EqualityPredicate, NoApplicationUnit, ReadOnly,
};

use crate::model::BankPrincipalId;
use crate::schema::{
    BankSchema, EstateBeneficiary, EstateCase, EstateDeceased, Principal, PrincipalIdentity,
    PrincipalIdentityField,
};

use super::customer_disclosure::EstateCustomerDisclosureQuery;

pub(super) struct CustomerRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(CustomerRelationSlot => "CustomerRelationSlot");
pub(super) struct CustomerIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(CustomerIdentitySlot => "CustomerIdentitySlot");
pub(super) struct BeneficiariesRelationSlot;
worth_query_decl::facade::worth_query_portable_type!(BeneficiariesRelationSlot => "BeneficiariesRelationSlot");
pub(super) struct BeneficiaryIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(BeneficiaryIdentitySlot => "BeneficiaryIdentitySlot");

pub(super) fn estate_customer() -> ApplicationQueryResultRelationRef<
    EstateCustomerDisclosureQuery,
    CustomerRelationSlot,
    BankSchema,
    EstateDeceased,
    EstateCase,
    Principal,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("customer", EstateDeceased::reference())
}

pub(super) fn estate_beneficiaries() -> ApplicationQueryResultRelationRef<
    EstateCustomerDisclosureQuery,
    BeneficiariesRelationSlot,
    BankSchema,
    EstateBeneficiary,
    Principal,
    EstateCase,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("beneficiaries", EstateBeneficiary::reference())
}

pub(super) fn customer_identity() -> ApplicationQueryResultFieldRef<
    EstateCustomerDisclosureQuery,
    CustomerIdentitySlot,
    BankSchema,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("customer_identity", PrincipalIdentityField::reference())
}

pub(super) fn beneficiary_identity() -> ApplicationQueryResultFieldRef<
    EstateCustomerDisclosureQuery,
    BeneficiaryIdentitySlot,
    BankSchema,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("beneficiary_identity", PrincipalIdentityField::reference())
}
