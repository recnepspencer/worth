//! Domain rows, binding ownership, and tuple membership for this fixture.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct IdentitySchema;
impl Fragment for IdentitySchema {
    const NAME: &'static str = "IdentitySchema";
    type Bindings = Bindings;
    type Members = (
        ExternalMapping,
        Principal,
        IdentityBinding,
        PrincipalQuery,
        PrincipalQueryBinding,
        result_only::ResultOnlyQueryBinding,
        resource::Assessment<resource::AssessmentResultBinding>,
    );
}
impl ApplicationSchema for IdentitySchema {
    const OWNER: &'static str = "identity_installation_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl IdentitySchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for Bindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
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
impl Principal {
    pub fn reference() -> ApplicationEntityRef<IdentitySchema, Self> {
        entity(IdentitySchema, Self)
    }
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}

pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for Bindings {
    fn record() -> impl Shape<ExternalIdentityValue, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
        .equality_queryable(true)
        .build(
            |external_identity_field, mapping_status_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status_field,
                })
            },
        )
    }
}
pub struct PrincipalIdentity;
impl Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = PrincipalIdentityValue;
}

pub struct PrincipalIdentityValue {
    pub principal_identity_field: u64,
}
impl Record<PrincipalIdentityValue> for Bindings {
    fn record() -> impl Shape<PrincipalIdentityValue, Self> {
        field("PrincipalIdentityField", |row: &PrincipalIdentityValue| {
            row.principal_identity_field
        })
        .equality_queryable(true)
        .read_only()
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalIdentityValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub type ExternalIdentityField = FieldSelection<
    IdentitySchema,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type PrincipalIdentityField = FieldSelection<IdentitySchema, Principal, PrincipalIdentity, u64>;
pub type MappingStatusField = FieldSelection<
    IdentitySchema,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<IdentitySchema, Self, ExternalMapping, Principal> {
        relation(IdentitySchema, ExternalMapping, Self)
    }
}

impl Scalar<String> for Bindings {
    type Binding = StringApplicationValueBinding;
}
impl Registration<IdentitySchema> for IdentityBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<IdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<IdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}
impl Registration<IdentitySchema> for PrincipalQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<IdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<IdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(principal_query_definition()))
    }
}
impl Registration<IdentitySchema> for PrincipalQueryBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<IdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<IdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}
impl Registration<IdentitySchema> for result_only::ResultOnlyQueryBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<IdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<IdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}
impl Registration<IdentitySchema> for resource::Assessment<resource::AssessmentResultBinding> {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<IdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<IdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(resource::with_authorization(builder).application_resource(resource::definition()))
    }
}
