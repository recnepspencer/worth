use crate::application_schema::{
    fragment::{self, *},
    *,
};
use crate::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};

pub struct IdentitySchema;
impl Fragment for IdentitySchema {
    const NAME: &'static str = "IdentitySchema";
    type Bindings = NoBindings;
    type Members = (IdentityModel, Contracts);
}
impl ApplicationSchema for IdentitySchema {
    const OWNER: &'static str = "identity_test";
    const NAME: &'static str = "IdentitySchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub struct IdentityModel;
impl Fragment for IdentityModel {
    const NAME: &'static str = "identity_model";
    type Bindings = IdentityBindings;
    type Members = (ExternalMapping, Principal);
}
pub struct IdentityBindings;
impl BindingSet for IdentityBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for IdentityBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for IdentityBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Scalar<u64> for IdentityBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = ();
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = MappingRow;
}
pub struct PrincipalIdentity;
impl Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = IdentityRow;
}
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
pub struct MappingRow {
    identity: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl Record<MappingRow> for IdentityBindings {
    fn record() -> impl Shape<MappingRow, Self> {
        field("identity", |row: &MappingRow| row.identity.clone())
            .equality_queryable(true)
            .and("status", |row: &MappingRow| row.status)
            .equality_queryable(true)
            .build(|identity, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingRow { identity, status })
            })
    }
}
pub struct IdentityBinding;
type IdentityField = FieldSelection<
    IdentityModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
type StatusField = FieldSelection<
    IdentityModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
type PrincipalField = FieldSelection<IdentityModel, Principal, PrincipalIdentity, u64>;
pub struct Contracts;
impl Registration<IdentitySchema> for Contracts {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<IdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<IdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(ApplicationPrincipalBindingRef::<
            IdentitySchema,
            IdentityBinding,
            ExternalMapping,
            Principal,
            u64,
            U64ApplicationValueBinding,
        >::from_requirements(
            "IdentityBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    IdentityField::at("identity"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    StatusField::at("status"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(relation(
                    IdentityModel,
                    ExternalMapping,
                    MappingTarget,
                )),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalField::at("value"),
                ),
            },
        )))
    }
}
#[test]
fn typed_principal_binding_enters_canonical_schema_members() {
    let declaration = IdentitySchema::declaration().unwrap();
    assert!(declaration.erased().members().iter().any(|member| matches!(member, ApplicationSchemaMember::PrincipalBinding { binding, .. } if binding == "IdentityBinding")));
}

fn refused(mut alter: impl FnMut(&mut Vec<ApplicationSchemaMember>)) {
    let mut members = IdentitySchema::declaration()
        .unwrap()
        .erased()
        .members()
        .to_vec();
    alter(&mut members);
    let denial = ApplicationSchemaDeclarationBuilder::<IdentitySchema>::from_test_members(members)
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::MissingPrincipalBindingDependency
    );
}
#[test]
fn forged_principal_binding_dependencies_fail_closed() {
    refused(|members| {
        for member in members {
            if let ApplicationSchemaMember::PrincipalBinding { status_field, .. } = member {
                *status_field = "missing".into();
            }
        }
    });
}
#[test]
fn principal_binding_rejects_wrong_field_type_and_write_posture() {
    for defect in ["identity_type", "identity_write", "status_write"] {
        refused(|members| {
            for member in members {
                if let ApplicationSchemaMember::Field {
                    field,
                    scalar_family,
                    value_type,
                    writable,
                    ..
                } = member
                {
                    match (defect, field.as_str()) {
                        ("identity_type", "identity") => {
                            *scalar_family = worth_foundational::facade::ScalarAspectType::UInt64;
                            *value_type = "worth.rust.u64".into();
                        }
                        ("identity_write", "identity") => *writable = true,
                        ("status_write", "status") => *writable = false,
                        _ => {}
                    }
                }
            }
        });
    }
}
#[test]
fn principal_binding_rejects_reversed_mapping_target_relation() {
    refused(|members| {
        for member in members {
            if let ApplicationSchemaMember::Relation { from, to, .. } = member {
                std::mem::swap(from, to);
            }
        }
    });
}
#[test]
fn principal_binding_rejects_mutable_target_principal_identity() {
    refused(|members| {
        for member in members {
            if let ApplicationSchemaMember::Field {
                entity, writable, ..
            } = member
                && entity == "Principal"
            {
                *writable = true;
            }
        }
    });
}
#[test]
fn principal_binding_rejects_wrong_target_principal_identity_type() {
    refused(|members| {
        for member in members {
            if let ApplicationSchemaMember::Field {
                entity,
                scalar_family,
                value_type,
                ..
            } = member
                && entity == "Principal"
            {
                *scalar_family = worth_foundational::facade::ScalarAspectType::String;
                *value_type = "worth.rust.string".into();
            }
        }
    });
}

pub struct IdentityRow(u64);
impl fragment::Record<IdentityRow> for IdentityBindings {
    fn record() -> impl fragment::Shape<IdentityRow, Self> {
        fragment::field("value", |row: &IdentityRow| row.0)
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IdentityRow(value))
            })
    }
}
