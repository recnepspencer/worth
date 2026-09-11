use worth_query_decl::facade::application_query::{
    ApplicationQueryResultShapeBuilder, TypedApplicationQueryResultShape,
};

use crate::{
    reads::EstateGovernanceContext,
    schema::{
        Account, BankSchema, Branch, CapabilityGrant, EmergencyAccess, EmployeeAssignment,
        EstateCase, Institution, MandatoryReview, Principal,
    },
};

use super::{governance::EstateGovernanceQuery, governance_fields::*, governance_relations::*};

type Shape<Entity> = ApplicationQueryResultShapeBuilder<
    BankSchema,
    EstateGovernanceQuery,
    Entity,
    (),
    crate::queries::UnitQueryResultBinding,
>;

pub(super) fn governance_shape() -> TypedApplicationQueryResultShape<
    BankSchema,
    EstateGovernanceQuery,
    EstateCase,
    EstateGovernanceContext,
    super::governance::EstateGovernanceQueryResultBinding,
> {
    let assignment = assignment_shape();
    let capability = capability_shape();
    ApplicationQueryResultShapeBuilder::new(EstateCase::reference())
        .field(estate_id())
        .field(estate_stage())
        .relation(estate_beneficiaries(), principal_shape(beneficiary()))
        .relation(estate_assignments(), assignment)
        .relation(estate_capabilities(), capability)
        .build()
}

fn assignment_shape() -> Shape<EmployeeAssignment> {
    ApplicationQueryResultShapeBuilder::new(EmployeeAssignment::reference())
        .field(assignment_id())
        .field(assignment_role())
        .relation(
            assignment_principal(),
            principal_shape(assignment_principal_identity()),
        )
}

fn capability_shape() -> Shape<CapabilityGrant> {
    ApplicationQueryResultShapeBuilder::new(CapabilityGrant::reference())
        .field(capability_id())
        .field(capability_operation())
        .field(capability_purpose())
        .optional_field(capability_field())
        .optional_field(capability_amount())
        .field(capability_valid_from())
        .field(capability_valid_through())
        .field(capability_delegation())
        .field(capability_workflow())
        .field(capability_status())
        .relation(
            capability_grantee(),
            principal_shape(capability_grantee_identity()),
        )
        .relation(
            capability_grantor(),
            principal_shape(capability_grantor_identity()),
        )
        .relation(capability_account(), account_shape())
        .relation(capability_institution(), institution_shape())
        .relation(capability_branch(), branch_shape())
        .relation(capability_parent(), parent_shape())
        .relation(capability_emergencies(), emergency_shape())
}

fn emergency_shape() -> Shape<EmergencyAccess> {
    ApplicationQueryResultShapeBuilder::new(EmergencyAccess::reference())
        .field(emergency_id())
        .field(emergency_reason())
        .field(emergency_status())
        .field(emergency_issued_at())
        .field(emergency_expires_at())
        .relation(
            emergency_requester(),
            principal_shape(emergency_requester_identity()),
        )
        .relation(
            emergency_approver(),
            principal_shape(emergency_approver_identity()),
        )
        .relation(emergency_review(), review_shape())
}

fn review_shape() -> Shape<MandatoryReview> {
    ApplicationQueryResultShapeBuilder::new(MandatoryReview::reference())
        .field(review_id())
        .field(review_kind())
        .field(review_status())
        .relation(
            review_estate(),
            Shape::<EstateCase>::new(EstateCase::reference()).field(review_estate_identity()),
        )
        .relation(
            review_reviewer(),
            principal_shape(review_reviewer_identity()),
        )
}

fn account_shape() -> Shape<Account> {
    ApplicationQueryResultShapeBuilder::new(Account::reference())
        .field(capability_account_identity())
}

fn institution_shape() -> Shape<Institution> {
    ApplicationQueryResultShapeBuilder::new(Institution::reference())
        .field(capability_institution_identity())
}

fn branch_shape() -> Shape<Branch> {
    ApplicationQueryResultShapeBuilder::new(Branch::reference()).field(capability_branch_identity())
}

fn parent_shape() -> Shape<CapabilityGrant> {
    ApplicationQueryResultShapeBuilder::new(CapabilityGrant::reference())
        .field(capability_parent_identity())
}

fn principal_shape<Slot: worth_query_decl::facade::portable_identity::WorthQueryPortableType>(
    selector: worth_query_decl::facade::application_query::ApplicationQueryResultFieldRef<
        EstateGovernanceQuery,
        Slot,
        BankSchema,
        Principal,
        crate::schema::PrincipalIdentity,
        crate::schema::PrincipalIdentityField,
        crate::model::BankPrincipalId,
        worth_query_decl::facade::application_schema::ReadOnly,
        worth_query_decl::facade::application_schema::EqualityPredicate,
        worth_query_decl::facade::application_schema::NoApplicationUnit,
    >,
) -> Shape<Principal> {
    ApplicationQueryResultShapeBuilder::new(Principal::reference()).field(selector)
}
