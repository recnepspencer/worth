use crate::application::{WorthQueryCapabilityFamily, WorthQueryDomainEntryMarker};
use crate::domain_installation::{
    WorthQueryDomainIdentityDeclaration, WorthQueryDomainIdentityName,
    WorthQueryDomainIdentityNamespace, WorthQueryDomainPackage, WorthQueryDomainSemanticVersion,
};
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_declaration::{
    worth_query_application_schema, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_principal_binding, worth_query_relation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PrimaryGraphDomain;

impl WorthQueryDomainEntryMarker for PrimaryGraphDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.primary-graph"
    }

    fn display_name(&self) -> &'static str {
        "Primary Graph Composition"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[]
    }
}

worth_query_application_schema! {
    pub(super) schema PrimaryGraphCompositionSchema {
        owner: "WORTH.tests.primary-graph",
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

worth_query_entity!(pub(super) ExternalMapping for PrimaryGraphCompositionSchema);
worth_query_entity!(pub(super) Principal for PrimaryGraphCompositionSchema);
worth_query_aspect!(pub(super) ExternalIdentity for PrimaryGraphCompositionSchema,
    ExternalMapping; identity = AspectIdentity(0x91611054), revision = AspectContractRevision(1),);
worth_query_field!(
    pub(super) ExternalIdentityField for PrimaryGraphCompositionSchema,
    ExternalMapping,
    ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_aspect!(pub(super) PrincipalIdentity for PrimaryGraphCompositionSchema,
    Principal; identity = AspectIdentity(0x91611055), revision = AspectContractRevision(1),);
worth_query_field!(
    pub(super) PrincipalIdentityField for PrimaryGraphCompositionSchema,
    Principal,
    PrincipalIdentity:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub(super) MappingStatusField for PrimaryGraphCompositionSchema,
    ExternalMapping,
    ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_relation!(
    pub(super) MappingTarget in PrimaryGraphCompositionSchema,
    ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub(super) IdentityBinding in PrimaryGraphCompositionSchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);

pub(super) fn primary_graph_domain_package() -> WorthQueryDomainPackage<PrimaryGraphDomain> {
    WorthQueryDomainPackage::declare(
        PrimaryGraphDomain,
        WorthQueryDomainIdentityDeclaration::new(
            WorthQueryDomainIdentityNamespace::new("WORTH.tests")
                .expect("test namespace should admit"),
            WorthQueryDomainIdentityName::new("primary-graph")
                .expect("test domain name should admit"),
            WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .application_schema(
        PrimaryGraphCompositionSchema::declaration().expect("test schema should declare"),
    )
}
