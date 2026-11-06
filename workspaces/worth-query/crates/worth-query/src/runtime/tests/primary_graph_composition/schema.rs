#[path = "schema/records.rs"]
mod storage_model;
use crate::application::{WorthQueryCapabilityFamily, WorthQueryDomainEntryMarker};
use crate::domain_installation::{
    WorthQueryDomainIdentityDeclaration, WorthQueryDomainIdentityName,
    WorthQueryDomainIdentityNamespace, WorthQueryDomainPackage, WorthQueryDomainSemanticVersion,
};
pub use storage_model::*;
use worth_query_declaration::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
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

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub(super) struct PrimaryGraphCompositionSchema;
impl Fragment for PrimaryGraphCompositionSchema {
    const NAME: &'static str = "PrimaryGraphCompositionSchema";
    type Bindings = NoBindings;
    type Members = (PrincipalRecords, Contracts);
}
impl ApplicationSchema for PrimaryGraphCompositionSchema {
    const OWNER: &'static str = "WORTH.tests.primary-graph";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl PrimaryGraphCompositionSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<PrimaryGraphCompositionSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<PrimaryGraphCompositionSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<PrimaryGraphCompositionSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.principal_binding(IdentityBinding::reference()))
    }
}

pub(super) struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        PrimaryGraphCompositionSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "IdentityBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

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
