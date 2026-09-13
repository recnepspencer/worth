use std::any::TypeId;

use crate::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationPrincipalScope,
    ApplicationMutationScopeResolution,
};
use crate::application_schema::{ApplicationSchemaMember, ApplicationStructuredValueBinding};
use crate::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenameInput {
    account_id: u64,
}

struct RenameInputBinding;

impl ApplicationStructuredValueBinding for RenameInputBinding {
    type Value = RenameInput;
    const IDENTITY_NAME: &'static str = "worth.query.test.rename-input.v1";

    fn validate(
        _: &Self::Value,
    ) -> Result<(), crate::application_schema::ApplicationValueValidationDenial> {
        Ok(())
    }
}

struct RenameDecision;
struct RenameDenial;
crate::worth_query_structured_value_binding!(RenameDenialBinding for RenameDenial {
    identity: "worth.query.test.rename-denial.v1"
});
struct RenameResult;
struct RenameOutputs;

crate::worth_query_structured_value_binding!(
    RenameResultBinding for RenameResult {
        identity: "worth.query.test.rename-result.v1"
    }
);

crate::worth_query_entity!(ExternalMapping for MutationSchema);
crate::worth_query_entity!(Account for MutationSchema);
crate::worth_query_aspect!(
    ExternalIdentity for MutationSchema, ExternalMapping;
    identity = AspectIdentity(0x9174_0001),
    revision = AspectContractRevision(1),
);
crate::worth_query_aspect!(
    AccountIdentity for MutationSchema, Account;
    identity = AspectIdentity(0x9174_0002),
    revision = AspectContractRevision(1),
);
crate::worth_query_field!(
    ExternalIdentityField for MutationSchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding,
    read_only, equality
);

impl super::ApplicationMutationOutputContract<MutationSchema> for RenameOutputs {
    const ROLES: &'static [super::ApplicationMutationOutputRoleDescriptor] = &[
        super::ApplicationMutationOutputRoleDescriptor::for_entity::<MutationSchema, Account>(
            "renamed-account",
            super::ApplicationMutationOutputPosture::Preserve,
        ),
    ];
}
crate::worth_query_field!(
    MappingStatusField for MutationSchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding,
    read_write, equality
);
crate::worth_query_field!(
    AccountIdField for MutationSchema, Account, AccountIdentity:
    u64 => crate::application_schema::U64ApplicationValueBinding, read_only, equality
);
crate::worth_query_relation!(
    MappingTarget in MutationSchema,
    ExternalMapping => Account;
    integrity = same_context_unbounded_retain_dangling
);
crate::worth_query_principal_binding!(
    PrincipalBinding in MutationSchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Account,
        principal_identity: AccountIdField
    }
);
crate::worth_query_operation!(
    RenameOperation for MutationSchema,
    input RenameInputBinding
);

fn rename_scope(input: &RenameInput) -> u64 {
    input.account_id
}

fn rename_key_identity(key: &u64) -> [u8; 32] {
    let mut identity = [0; 32];
    identity[..8].copy_from_slice(&key.to_be_bytes());
    identity
}

fn rename_input_identity(input: &RenameInput) -> [u8; 32] {
    let mut identity = [0; 32];
    identity[..8].copy_from_slice(&input.account_id.to_be_bytes());
    identity
}

crate::worth_query_mutation_binding!(
    RenameBinding for RenameInput, schema MutationSchema,
    identity "worth.query.test.rename-binding.v1",
    input RenameInputBinding,
    operation RenameOperation,
    result RenameResultBinding,
    idempotency u64, identity "worth.query.test.rename-idempotency.v1",
        key_identity rename_key_identity, input_identity rename_input_identity,
    decision RenameDecision, denial RenameDenialBinding,
    handler identity "worth.query.test.rename-handler.v1",
    outputs RenameOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Account,
        principal_identity u64, identity_binding crate::application_schema::U64ApplicationValueBinding,
    scope Account, AccountIdentity, AccountIdField, u64,
        crate::application_schema::ReadOnly, crate::application_schema::NoApplicationUnit,
    field AccountIdField::reference(),
    value rename_scope,
    candidates creates 0, deletes 0, links 0, unlinks 0, writes 1, emits 1,
    resources retained_representation_bytes 128, validator_work 4
);

crate::worth_query_application_schema! {
    schema MutationSchema {
        owner: mutation_binding_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(ExternalMapping::reference())
                .entity(Account::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Account::reference(), AccountIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Account::reference(), AccountIdField::reference())
                .relation(
                    MappingTarget::reference(),
                    ExternalMapping::reference(),
                    Account::reference(),
                )
                .principal_binding(PrincipalBinding::reference())
                .operation(
                    RenameOperation::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .application_mutation_binding::<RenameBinding>()
        }
    }
}

#[test]
fn mutation_binding_registers_exact_handler_scope_and_candidate_metadata() {
    let declaration = MutationSchema::declaration().expect("mutation schema should declare");
    let descriptor = declaration
        .member_provenance()
        .mutation_bindings()
        .first()
        .expect("mutation binding should be registered");

    assert_eq!(descriptor.identity(), "worth.query.test.rename-binding.v1");
    assert_eq!(descriptor.operation_name(), "RenameOperation");
    assert_eq!(
        descriptor.result_identity().as_str(),
        "worth.query.test.rename-result.v1"
    );
    assert_eq!(descriptor.result_type(), TypeId::of::<RenameResult>());
    assert_eq!(
        descriptor.handler().identity(),
        "worth.query.test.rename-handler.v1"
    );
    assert_eq!(
        descriptor.idempotency().identity(),
        "worth.query.test.rename-idempotency.v1"
    );
    assert_eq!(descriptor.idempotency().key_type(), TypeId::of::<u64>());
    assert_eq!(
        descriptor.handler().decision_type(),
        TypeId::of::<RenameDecision>()
    );
    assert_eq!(
        descriptor.handler().denial_type(),
        TypeId::of::<RenameDenial>()
    );
    assert_eq!(
        descriptor.output_contract_type(),
        TypeId::of::<RenameOutputs>()
    );
    assert_eq!(
        descriptor.output_roles(),
        <RenameOutputs as super::ApplicationMutationOutputContract<MutationSchema>>::ROLES
    );
    assert_eq!(descriptor.scope_entity(), "Account");
    assert_eq!(descriptor.scope().field().locus().field(), "AccountIdField");
    assert_eq!(descriptor.candidates().cardinality().maximum_writes(), 1);
    assert_eq!(descriptor.candidates().cardinality().maximum_emits(), 1);
    assert_eq!(
        descriptor
            .candidates()
            .resources()
            .maximum_retained_representation_bytes(),
        128
    );
    assert!(declaration
        .member_provenance()
        .admits_mutation_binding_operation(descriptor));
    assert!(declaration.erased().members().iter().any(|member| matches!(
        member,
        ApplicationSchemaMember::Operation { operation, .. }
            if operation == "RenameOperation"
    )));
}

#[test]
fn input_field_scope_resolution_retains_input_and_principal_affinity() {
    let intent = RenameInput { account_id: 41 };
    let operation = intent.operation();
    let (field, resolved) = intent.scope_binding().into_field_parts(&900_u64);

    assert_eq!(operation.name(), "RenameOperation");
    assert_eq!(field.entity(), "Account");
    assert_eq!(resolved, 41);
    assert_eq!(
        RenameBinding::descriptor().input_binding_type(),
        TypeId::of::<RenameInputBinding>()
    );
}

#[test]
fn principal_scope_resolution_uses_the_borrowed_principal_identity() {
    let scope = ApplicationMutationPrincipalScope::new(AccountIdField::reference());
    let (field, resolved) = scope.into_field_parts(&73_u64);

    assert_eq!(field.entity(), "Account");
    assert_eq!(resolved, 73);
}

#[test]
fn declaration_description_survives_portable_readmission_and_binds_denial_identity() {
    use crate::application_schema::{
        validate_portable_application_schema_freshly, WorthQueryPortableApplicationSchemaRecord,
    };
    let source = MutationSchema::declaration().unwrap();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(source.erased());
    let restored = validate_portable_application_schema_freshly(portable.clone()).unwrap();
    assert_eq!(source.identity(), restored.identity());
    let description = restored
        .members()
        .iter()
        .find_map(|member| match member {
            ApplicationSchemaMember::ApplicationMutation { description } => Some(description),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        description.input_identity().as_str(),
        "worth.query.test.rename-input.v1"
    );
    assert_eq!(
        description.result_identity().as_str(),
        "worth.query.test.rename-result.v1"
    );
    assert_eq!(
        description.denial_identity().as_str(),
        "worth.query.test.rename-denial.v1"
    );
    assert_eq!(description.scope().field, "AccountIdField");
    assert_eq!(description.output_roles()[0].name, "renamed-account");

    let mut changed = portable.into_parts();
    for member in &mut changed.members {
        if let ApplicationSchemaMember::ApplicationMutation { description } = member {
            let mut parts = description.clone().into_parts();
            parts.denial_identity =
                crate::portable_identity::WorthQueryPortableTypeIdentity::declared(
                    "worth.query.test.other-denial.v1",
                );
            *description = super::ApplicationMutationDescription::from_untrusted_parts(parts);
        }
    }
    changed.members.sort();
    let changed = validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(changed),
    )
    .unwrap();
    assert_ne!(restored.identity(), changed.identity());
}

#[test]
fn portable_mutation_description_rejects_missing_scope_and_duplicate_binding() {
    use crate::application_schema::{
        validate_portable_application_schema_freshly, ApplicationSchemaDeclarationDenial,
        WorthQueryPortableApplicationSchemaRecord,
    };
    let source = MutationSchema::declaration().unwrap();
    let portable = WorthQueryPortableApplicationSchemaRecord::project(source.erased());
    let mut missing_scope = portable.clone().into_parts();
    for member in &mut missing_scope.members {
        if let ApplicationSchemaMember::ApplicationMutation { description } = member {
            let mut parts = description.clone().into_parts();
            parts.scope.field = "MissingScopeField".to_owned();
            *description = super::ApplicationMutationDescription::from_untrusted_parts(parts);
        }
    }
    missing_scope.members.sort();
    assert!(matches!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(missing_scope)
        ),
        Err(ApplicationSchemaDeclarationDenial::MissingApplicationMutationDependency)
    ));
    let mut duplicate = portable.into_parts();
    let member = duplicate
        .members
        .iter()
        .find(|member| matches!(member, ApplicationSchemaMember::ApplicationMutation { .. }))
        .unwrap()
        .clone();
    duplicate.members.push(member);
    duplicate.members.sort();
    assert_eq!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(duplicate)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::DuplicateMember
    );
}
