//! The installed world keeps the SDK's ordinary principal-binding requirement.
use super::*;
use worth_query_host::facade::declaration::authentication::*;

pub struct Authentication;
impl Fragment for Authentication {
    const NAME: &'static str = "authentication";
    type Bindings = Bindings;
    type Members = (Mapping, Principal, Binding);
}
pub struct Mapping;
impl Entity for Mapping {
    const NAME: &'static str = "mapping";
    type Aspects = (MappingIdentity,);
    type Relations = (MapsTo,);
}
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = ();
}
pub struct MapsTo;
impl Relation for MapsTo {
    const NAME: &'static str = "maps_to";
    type To = Principal;
}
pub struct MappingRow {
    external: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
pub struct PrincipalRow(u64);
pub struct MappingIdentity;
impl Aspect for MappingIdentity {
    const NAME: &'static str = "mapping_identity";
    type Row = MappingRow;
}
pub struct PrincipalIdentity;
impl Aspect for PrincipalIdentity {
    const NAME: &'static str = "principal_identity";
    type Row = PrincipalRow;
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for Bindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Record<MappingRow> for Bindings {
    fn record() -> impl Shape<MappingRow, Self> {
        field("external", |row: &MappingRow| row.external.clone())
            .equality_queryable(true)
            .unique()
            .and("status", |row: &MappingRow| row.status)
            .build(|external, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingRow { external, status })
            })
    }
}
impl Record<PrincipalRow> for Bindings {
    fn record() -> impl Shape<PrincipalRow, Self> {
        field("id", |row: &PrincipalRow| row.0)
            .equality_queryable(true)
            .read_only()
            .build(|id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow(id))
            })
    }
}
pub struct Binding;
pub fn binding<S: ApplicationSchema>(
) -> ApplicationPrincipalBindingRef<S, Binding, Mapping, Principal, u64, U64ApplicationValueBinding>
{
    ApplicationPrincipalBindingRef::from_requirements(
        "identity",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_record(
                record_field(Authentication, Mapping, MappingIdentity, "external"),
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_record(
                record_field(Authentication, Mapping, MappingIdentity, "status"),
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(relation(
                Authentication,
                Mapping,
                MapsTo,
            )),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_record(
                Authentication,
                record_field(Authentication, Principal, PrincipalIdentity, "id"),
            ),
        },
    )
}
impl<S: ApplicationSchema> Registration<S> for Binding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        Ok(builder.principal_binding(binding()))
    }
}
