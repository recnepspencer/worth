use std::any::TypeId;

use bank_domain::{
    model::{AccountId, AccountName, BankPrincipalId, InstitutionId},
    proposals::{BankIdempotencyKey, BankProposalDenial},
    schema::{
        BankSchema, CreatePersonalAccount, CreatePersonalAccountDecision,
        CreatePersonalAccountMutationBinding, CreatePersonalAccountOperation,
        CreatePersonalAccountResult, CreatePersonalAccountResultBinding,
    },
};
use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationIntent,
        ApplicationMutationScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};

#[test]
fn personal_account_creation_binding_declares_its_complete_fixed_shape() {
    let declaration = BankSchema::declaration().expect("bank schema should declare");
    let descriptor = declaration
        .member_provenance()
        .mutation_bindings()
        .iter()
        .find(|binding| binding.operation_name() == "CreatePersonalAccountOperation")
        .expect("personal account creation mutation binding should be installed");

    assert_eq!(
        descriptor.identity(),
        "bank.operation.create-personal-account.mutation-binding.v1"
    );
    assert_eq!(
        descriptor.result_identity().as_str(),
        "bank.operation.create-personal-account.result.v1"
    );
    assert_eq!(
        descriptor.result_type(),
        TypeId::of::<CreatePersonalAccountResult>()
    );
    assert_eq!(
        descriptor.handler().identity(),
        "bank.operation.create-personal-account.handler.v1"
    );
    assert_eq!(
        descriptor.handler().decision_type(),
        TypeId::of::<CreatePersonalAccountDecision>()
    );
    assert_eq!(
        descriptor.handler().denial_type(),
        TypeId::of::<BankProposalDenial>()
    );
    assert_eq!(
        descriptor.description().denial_identity().as_str(),
        "bank.operation.create-personal-account.denial.v1"
    );
    assert_eq!(
        descriptor.idempotency().identity(),
        "bank.application-mutation-client-key.v1"
    );
    assert_eq!(
        descriptor.idempotency().key_type(),
        TypeId::of::<BankIdempotencyKey>()
    );
    assert_eq!(descriptor.principal().name(), "BankPrincipalBinding");
    assert_eq!(descriptor.scope_entity(), "Institution");
    assert_eq!(
        descriptor.scope().field().locus().field(),
        "InstitutionIdentityField"
    );

    let cardinality = descriptor.candidates().cardinality();
    assert_eq!(cardinality.maximum_creates(), 1);
    assert_eq!(cardinality.maximum_deletes(), 0);
    assert_eq!(cardinality.maximum_links(), 2);
    assert_eq!(cardinality.maximum_unlinks(), 0);
    assert_eq!(cardinality.maximum_writes(), 5);
    assert_eq!(cardinality.maximum_emits(), 0);
    assert_eq!(
        descriptor
            .candidates()
            .resources()
            .maximum_retained_representation_bytes(),
        1392
    );
}

#[test]
fn personal_account_creation_intent_carries_institution_scope_and_typed_result() {
    let institution = InstitutionId::new(7).unwrap();
    let input = CreatePersonalAccount {
        institution,
        owner: BankPrincipalId::new(11).unwrap(),
        display_name: AccountName::new("Household").unwrap(),
    };
    let operation = input.operation();
    let (_, resolved_scope) = input
        .scope_binding()
        .into_field_parts(&BankPrincipalId::new(11).unwrap());

    assert_eq!(operation.name(), "CreatePersonalAccountOperation");
    assert_eq!(resolved_scope, institution);
    assert_eq!(
        TypeId::of::<<CreatePersonalAccountMutationBinding as ApplicationMutationBinding<
            BankSchema,
        >>::Operation>(),
        TypeId::of::<CreatePersonalAccountOperation>()
    );
    assert_eq!(
        TypeId::of::<
            <CreatePersonalAccountResultBinding as ApplicationStructuredValueBinding>::Value,
        >(),
        TypeId::of::<CreatePersonalAccountResult>()
    );
    let result = CreatePersonalAccountResult {
        account: AccountId::new(19).unwrap(),
    };
    assert_eq!(result.account, AccountId::new(19).unwrap());
}

#[test]
fn personal_account_creation_binding_canonicalizes_only_client_key_and_input() {
    let key = |text: &str| BankIdempotencyKey::new(text).unwrap();
    let key_identity = |text: &str| {
        *ApplicationMutationIdentities::<BankSchema, CreatePersonalAccountMutationBinding>::encode(
            &key(text),
            &personal_account_input(7, 11, "Household"),
        )
        .expect("key encodes")
        .key_identity()
    };
    assert_eq!(
        key_identity("open-household"),
        key_identity("open-household")
    );
    assert_ne!(key_identity("open-household"), key_identity("open-savings"));

    let input_identity = |institution, owner, name: &str| {
        *ApplicationMutationIdentities::<BankSchema, CreatePersonalAccountMutationBinding>::encode(
            &key("open-household"),
            &personal_account_input(institution, owner, name),
        )
        .expect("input encodes")
        .input_identity()
    };
    let baseline = input_identity(7, 11, "Household");
    assert_eq!(baseline, input_identity(7, 11, "Household"));
    assert_ne!(baseline, input_identity(8, 11, "Household"));
    assert_ne!(baseline, input_identity(7, 12, "Household"));
    assert_ne!(baseline, input_identity(7, 11, "Savings"));
}

fn personal_account_input(
    institution: u64,
    owner: u64,
    display_name: &str,
) -> CreatePersonalAccount {
    CreatePersonalAccount {
        institution: InstitutionId::new(institution).unwrap(),
        owner: BankPrincipalId::new(owner).unwrap(),
        display_name: AccountName::new(display_name).unwrap(),
    }
}
