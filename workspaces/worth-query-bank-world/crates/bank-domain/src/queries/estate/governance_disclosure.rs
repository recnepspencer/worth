use worth_query_decl::facade::application_query::{
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
};

use crate::{estate::RestrictedBankField, schema::ViewEstateAdministrationCapability};

use super::{governance_fields::*, governance_relations::*};

pub(super) fn governance_disclosure() -> ApplicationQueryDisclosureContract {
    let influence = ApplicationQueryInfluenceContract::forbid_all();
    let contract = ApplicationQueryDisclosureContract::governed_by(
        "estate-governance-context",
        ViewEstateAdministrationCapability::reference(),
    );
    let contract = disclose_estate(contract, &influence);
    let contract = disclose_assignment(contract, &influence);
    let contract = disclose_capability(contract, &influence);
    let contract = disclose_emergency(contract, &influence);
    disclose_review(contract, &influence)
}

fn disclose_estate(
    contract: ApplicationQueryDisclosureContract,
    influence: &ApplicationQueryInfluenceContract,
) -> ApplicationQueryDisclosureContract {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::GovernanceMetadata,
        )
    };
    contract
        .disclose_field_by(estate_id(), field(), influence.clone())
        .disclose_field_by(estate_stage(), field(), influence.clone())
        .disclose_relation_by(estate_beneficiaries(), field(), influence.clone())
        .disclose_field_by(beneficiary(), field(), influence.clone())
        .disclose_relation_by(estate_assignments(), field(), influence.clone())
        .disclose_relation_by(estate_capabilities(), field(), influence.clone())
}

fn disclose_assignment(
    contract: ApplicationQueryDisclosureContract,
    influence: &ApplicationQueryInfluenceContract,
) -> ApplicationQueryDisclosureContract {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::GovernanceMetadata,
        )
    };
    contract
        .disclose_field_by(assignment_id(), field(), influence.clone())
        .disclose_field_by(assignment_role(), field(), influence.clone())
        .disclose_relation_by(assignment_principal(), field(), influence.clone())
        .disclose_field_by(assignment_principal_identity(), field(), influence.clone())
}

fn disclose_capability(
    contract: ApplicationQueryDisclosureContract,
    influence: &ApplicationQueryInfluenceContract,
) -> ApplicationQueryDisclosureContract {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::GovernanceMetadata,
        )
    };
    contract
        .disclose_field_by(capability_id(), field(), influence.clone())
        .disclose_field_by(capability_operation(), field(), influence.clone())
        .disclose_field_by(capability_purpose(), field(), influence.clone())
        .disclose_optional_field_by(capability_field(), field(), influence.clone())
        .disclose_optional_field_by(capability_amount(), field(), influence.clone())
        .disclose_field_by(capability_valid_from(), field(), influence.clone())
        .disclose_field_by(capability_valid_through(), field(), influence.clone())
        .disclose_field_by(capability_delegation(), field(), influence.clone())
        .disclose_field_by(capability_workflow(), field(), influence.clone())
        .disclose_field_by(capability_status(), field(), influence.clone())
        .disclose_relation_by(capability_grantee(), field(), influence.clone())
        .disclose_field_by(capability_grantee_identity(), field(), influence.clone())
        .disclose_relation_by(capability_grantor(), field(), influence.clone())
        .disclose_field_by(capability_grantor_identity(), field(), influence.clone())
        .disclose_relation_by(capability_account(), field(), influence.clone())
        .disclose_field_by(capability_account_identity(), field(), influence.clone())
        .disclose_relation_by(capability_institution(), field(), influence.clone())
        .disclose_field_by(
            capability_institution_identity(),
            field(),
            influence.clone(),
        )
        .disclose_relation_by(capability_branch(), field(), influence.clone())
        .disclose_field_by(capability_branch_identity(), field(), influence.clone())
        .disclose_relation_by(capability_parent(), field(), influence.clone())
        .disclose_field_by(capability_parent_identity(), field(), influence.clone())
        .disclose_relation_by(capability_emergencies(), field(), influence.clone())
}

fn disclose_emergency(
    contract: ApplicationQueryDisclosureContract,
    influence: &ApplicationQueryInfluenceContract,
) -> ApplicationQueryDisclosureContract {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::GovernanceMetadata,
        )
    };
    contract
        .disclose_field_by(emergency_id(), field(), influence.clone())
        .disclose_field_by(emergency_reason(), field(), influence.clone())
        .disclose_field_by(emergency_status(), field(), influence.clone())
        .disclose_field_by(emergency_issued_at(), field(), influence.clone())
        .disclose_field_by(emergency_expires_at(), field(), influence.clone())
        .disclose_relation_by(emergency_requester(), field(), influence.clone())
        .disclose_field_by(emergency_requester_identity(), field(), influence.clone())
        .disclose_relation_by(emergency_approver(), field(), influence.clone())
        .disclose_field_by(emergency_approver_identity(), field(), influence.clone())
        .disclose_relation_by(emergency_review(), field(), influence.clone())
}

fn disclose_review(
    contract: ApplicationQueryDisclosureContract,
    influence: &ApplicationQueryInfluenceContract,
) -> ApplicationQueryDisclosureContract {
    let field = || {
        crate::schema::encoded_bank_value::<crate::schema::RestrictedBankFieldBinding>(
            RestrictedBankField::GovernanceMetadata,
        )
    };
    contract
        .disclose_field_by(review_id(), field(), influence.clone())
        .disclose_field_by(review_kind(), field(), influence.clone())
        .disclose_field_by(review_status(), field(), influence.clone())
        .disclose_relation_by(review_estate(), field(), influence.clone())
        .disclose_field_by(review_estate_identity(), field(), influence.clone())
        .disclose_relation_by(review_reviewer(), field(), influence.clone())
        .disclose_field_by(review_reviewer_identity(), field(), influence.clone())
}
