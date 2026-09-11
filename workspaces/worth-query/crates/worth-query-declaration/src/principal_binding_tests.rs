use crate::facade::application_schema::{
    ApplicationFieldRef, ApplicationPrincipalBindingRef, ApplicationPrincipalBindingRequirements,
    ApplicationPrincipalIdentityRequirement, ApplicationPrincipalMappingIdentityRequirement,
    ApplicationPrincipalMappingStatusRequirement, ApplicationPrincipalTargetRequirement,
    ApplicationRelationRef, ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
    EqualityPredicate, ReadOnly, ReadWrite, StringApplicationValueBinding,
    U64ApplicationValueBinding,
};
use crate::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
mod forged_binding_fixture;
use forged_binding_fixture::forged_binding;

worth_query_application_schema! {
    pub schema IdentitySchema {
        owner: identity_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .relation(
                    MappingTarget::reference(),
                    ExternalMapping::reference(),
                    Principal::reference(),
                )
                .principal_binding(IdentityBinding::reference())
        }
    }
}

worth_query_entity!(pub ExternalMapping for IdentitySchema);
worth_query_entity!(pub Principal for IdentitySchema);
worth_query_aspect!(pub ExternalIdentity for IdentitySchema, ExternalMapping; identity = AspectIdentity(0x9161102d), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityField for IdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_aspect!(pub PrincipalIdentity for IdentitySchema, Principal; identity = AspectIdentity(0x9161102e), revision = AspectContractRevision(1),);
worth_query_field!(
    pub PrincipalIdentityField for IdentitySchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub MutablePrincipalIdentityField for IdentitySchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_write, equality
);
worth_query_field!(
    pub WrongPrincipalIdentityField for IdentitySchema, Principal, PrincipalIdentity:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub MappingStatusField for IdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_field!(
    pub MutableExternalIdentityField for IdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_write, equality
);
worth_query_field!(
    pub ImmutableMappingStatusField for IdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_only, equality
);
worth_query_relation!(
    pub MappingTarget in IdentitySchema,
    ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(
    pub ReversedMappingTarget in IdentitySchema,
    Principal => ExternalMapping; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub IdentityBinding in IdentitySchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);

#[test]
fn typed_principal_binding_enters_canonical_schema_members() {
    let declaration = IdentitySchema::declaration().unwrap();
    assert!(declaration.erased().members().iter().any(|member| matches!(
        member,
        ApplicationSchemaMember::PrincipalBinding { binding, .. }
            if binding == "IdentityBinding"
    )));
}

#[test]
fn forged_principal_binding_dependencies_fail_closed() {
    let forged = forged_binding::<u64>(
        "ExternalIdentityField",
        "MissingStatusField",
        "MappingTarget",
        "PrincipalIdentityField",
    );
    let denial = crate::facade::application_schema::ApplicationSchemaDeclarationBuilder::<
        IdentitySchema,
    >::for_schema()
    .entity(ExternalMapping::reference())
    .entity(Principal::reference())
    .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
    .field(
        ExternalMapping::reference(),
        ExternalIdentityField::reference(),
    )
    .field(
        ExternalMapping::reference(),
        MappingStatusField::reference(),
    )
    .aspect(Principal::reference(), PrincipalIdentity::reference())
    .field(Principal::reference(), PrincipalIdentityField::reference())
    .relation(
        MappingTarget::reference(),
        ExternalMapping::reference(),
        Principal::reference(),
    )
    .principal_binding(forged)
    .build()
    .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::MissingPrincipalBindingDependency
    );
}

#[test]
fn principal_binding_rejects_wrong_field_type_and_write_posture() {
    for case in [
        HostileBindingCase::new("ExternalIdentityField", "ExternalIdentityField"),
        HostileBindingCase::new("MutableExternalIdentityField", "MappingStatusField")
            .with_mutable_identity(),
        HostileBindingCase::new("ExternalIdentityField", "ImmutableMappingStatusField")
            .with_immutable_status(),
    ] {
        let denial = identity_declaration_with(case).unwrap_err();
        assert_eq!(
            denial,
            ApplicationSchemaDeclarationDenial::MissingPrincipalBindingDependency
        );
    }
}

#[test]
fn principal_binding_rejects_reversed_mapping_target_relation() {
    let denial = identity_declaration_with(
        HostileBindingCase::new("ExternalIdentityField", "MappingStatusField")
            .with_reversed_relation(),
    )
    .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::MissingPrincipalBindingDependency
    );
}

#[test]
fn principal_binding_rejects_mutable_target_principal_identity() {
    let forged = forged_binding::<u64>(
        "ExternalIdentityField",
        "MappingStatusField",
        "MappingTarget",
        "MutablePrincipalIdentityField",
    );
    let denial = identity_declaration_builder()
        .field(
            Principal::reference(),
            MutablePrincipalIdentityField::reference(),
        )
        .principal_binding(forged)
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::MissingPrincipalBindingDependency
    );
}

#[test]
fn principal_binding_rejects_wrong_target_principal_identity_type() {
    let forged = forged_binding::<String>(
        "ExternalIdentityField",
        "MappingStatusField",
        "MappingTarget",
        "PrincipalIdentityField",
    );
    let denial = identity_declaration_builder()
        .field(
            Principal::reference(),
            WrongPrincipalIdentityField::reference(),
        )
        .principal_binding(forged)
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::MissingPrincipalBindingDependency
    );
}

struct HostileBindingCase {
    identity_field: &'static str,
    status_field: &'static str,
    mutable_identity: bool,
    immutable_status: bool,
    reversed_relation: bool,
}

impl HostileBindingCase {
    const fn new(identity_field: &'static str, status_field: &'static str) -> Self {
        Self {
            identity_field,
            status_field,
            mutable_identity: false,
            immutable_status: false,
            reversed_relation: false,
        }
    }

    const fn with_mutable_identity(mut self) -> Self {
        self.mutable_identity = true;
        self
    }

    const fn with_immutable_status(mut self) -> Self {
        self.immutable_status = true;
        self
    }

    const fn with_reversed_relation(mut self) -> Self {
        self.reversed_relation = true;
        self
    }

    const fn target_relation(&self) -> &'static str {
        if self.reversed_relation {
            "ReversedMappingTarget"
        } else {
            "MappingTarget"
        }
    }
}

fn identity_declaration_with(
    case: HostileBindingCase,
) -> Result<
    crate::facade::application_schema::ApplicationSchemaDeclaration<IdentitySchema>,
    ApplicationSchemaDeclarationDenial,
> {
    let binding = forged_binding::<u64>(
        case.identity_field,
        case.status_field,
        case.target_relation(),
        "PrincipalIdentityField",
    );
    let builder = crate::facade::application_schema::ApplicationSchemaDeclarationBuilder::<
        IdentitySchema,
    >::for_schema()
    .entity(ExternalMapping::reference())
    .entity(Principal::reference())
    .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
    .aspect(Principal::reference(), PrincipalIdentity::reference())
    .field(Principal::reference(), PrincipalIdentityField::reference());
    let builder = if case.mutable_identity {
        builder.field(
            ExternalMapping::reference(),
            MutableExternalIdentityField::reference(),
        )
    } else {
        builder.field(
            ExternalMapping::reference(),
            ExternalIdentityField::reference(),
        )
    };
    let builder = if case.immutable_status {
        builder.field(
            ExternalMapping::reference(),
            ImmutableMappingStatusField::reference(),
        )
    } else {
        builder.field(
            ExternalMapping::reference(),
            MappingStatusField::reference(),
        )
    };
    let builder = if case.reversed_relation {
        builder.relation(
            ReversedMappingTarget::reference(),
            Principal::reference(),
            ExternalMapping::reference(),
        )
    } else {
        builder.relation(
            MappingTarget::reference(),
            ExternalMapping::reference(),
            Principal::reference(),
        )
    };
    builder.principal_binding(binding).build()
}

fn identity_declaration_builder(
) -> crate::facade::application_schema::ApplicationSchemaDeclarationBuilder<IdentitySchema> {
    crate::facade::application_schema::ApplicationSchemaDeclarationBuilder::<IdentitySchema>::for_schema()
        .entity(ExternalMapping::reference())
        .entity(Principal::reference())
        .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
        .aspect(Principal::reference(), PrincipalIdentity::reference())
        .field(
            ExternalMapping::reference(),
            ExternalIdentityField::reference(),
        )
        .field(
            ExternalMapping::reference(),
            MappingStatusField::reference(),
        )
        .relation(
            MappingTarget::reference(),
            ExternalMapping::reference(),
            Principal::reference(),
        )
}
