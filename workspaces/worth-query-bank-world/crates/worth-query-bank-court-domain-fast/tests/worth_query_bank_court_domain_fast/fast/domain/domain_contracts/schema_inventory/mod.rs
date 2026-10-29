pub(crate) use worth_query_bank_worlds::facade::domain::domain_contracts::schema_inventory::*;

pub(crate) mod source_contract;

#[test]
fn bank_manifest_matches_the_frozen_schema_world() {
    let declaration = BankSchema::declaration().unwrap();
    let members = declaration.erased().members();
    assert_entity_and_relation_inventory(members);
    assert_operation_inventory(members);
    assert_capability_inventory(members);
    assert_application_query_inventory(members);
    field_inventory::assert_field_and_governance_inventory(members);
    assert_account_creation_programs(members);
    assert_money_programs(members);
    assert_payment_and_authorization_programs(members);
    assert_eq!(
        declaration.erased().owner(),
        "WORTH.bank",
        "host package and application schema must share one canonical owner"
    );
    let binding = BankPrincipalBinding::reference();
    assert_eq!(binding.mapping_entity(), "ExternalPrincipalMapping");
    assert_eq!(binding.identity_field(), "ExternalIdentityKey");
    assert_eq!(binding.status_field(), "ExternalMappingStatus");
    assert_eq!(binding.target_relation(), "ExternalPrincipal");
    assert_eq!(binding.principal_entity(), "Principal");
    assert_eq!(binding.principal_identity_aspect(), "PrincipalIdentity");
    assert_eq!(binding.principal_identity_field(), "PrincipalIdentityField");
}

#[test]
fn send_projection_capability_does_not_widen_mandatory_commit_dependencies() {
    let declaration = BankSchema::declaration().unwrap();
    let members = declaration.erased().members();
    assert_decision_reads(
        members,
        "SendMoneyOperation",
        &[
            "field:Account/AccountState/AccountingRevision",
            "field:Account/AccountState/Status",
            "field:Account/Identity/AccountIdentity",
            "field:Principal/PrincipalIdentity/PrincipalIdentityField",
            "relation:PersonalOwner:Principal->Account",
        ],
    );
}

#[test]
fn revoke_projection_capability_does_not_widen_mandatory_commit_dependencies() {
    let declaration = BankSchema::declaration().unwrap();
    let members = declaration.erased().members();
    assert_decision_reads(
        members,
        "RevokeAccountAuthorizationOperation",
        &[
            "entity:AccountAuthorization",
            "field:Account/Identity/AccountIdentity",
            "field:AccountAuthorization/AuthorizationIdentity/AccountAuthorizationIdentity",
            "field:AccountAuthorization/AuthorizationScope/AuthorizationRole",
            "field:Principal/PrincipalIdentity/PrincipalIdentityField",
            "relation:AccountAuthorizedUser:Principal->AccountAuthorization",
            "relation:AuthorizationAccount:AccountAuthorization->Account",
        ],
    );
}
