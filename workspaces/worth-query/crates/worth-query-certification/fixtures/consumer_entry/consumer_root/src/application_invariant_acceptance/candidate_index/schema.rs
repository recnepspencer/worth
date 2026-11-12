//! The Candidate fixture's independent root owns its records and contracts.
use super::{boxes, directory, mutation, prefix, query};
use worth_query_decl::facade::application_query::ApplicationManagedIndexDefinition;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::{application_schema::*, authentication::*};
mod records;
pub use records::*;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Catalog;
impl Fragment for Catalog {
    const NAME: &'static str = "Catalog";
    type Bindings = NoBindings;
    type Members = (CatalogContribution,);
}
impl ApplicationSchema for Catalog {
    const OWNER: &'static str = "worth.query.consumer.managed-catalog";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl ApplicationSchemaComposition for Catalog {
    type Contributions = (CatalogContribution,);
}
pub struct CatalogContribution;
impl ApplicationSchemaContribution<Catalog> for CatalogContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new(
            "worth.query.consumer.managed-catalog.contribution.v1",
        );
    type Members = (
        CatalogRecords,
        boxes::access::grants::BoxGrantRecords,
        ContributionContracts,
    );
}
pub struct ContributionContracts;
impl Registration<Catalog> for ContributionContracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Catalog>, ApplicationSchemaDeclarationDenial>
    {
        Ok(directory::declare(mutation::declare(boxes::declare(
            schema
                .principal_binding(CatalogPrincipal::reference())
                .application_query(query::source_definition(true))
                .application_query_binding::<query::SourceBinding>()
                .application_query(query::prefix_definition())
                .application_query_binding::<query::PrefixBinding>()
                .application_query_binding::<query::PrefixAliasBinding>()
                .managed_index(ApplicationManagedIndexDefinition::<
                    Catalog,
                    prefix::PrefixIndex,
                >::new()),
        ))))
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogName(String);
impl From<String> for CatalogName {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for CatalogName {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}
impl From<CatalogName> for String {
    fn from(value: CatalogName) -> Self {
        value.0
    }
}
pub struct NameBinding;
impl ApplicationScalarValueBinding for NameBinding {
    type Value = CatalogName;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "managed-catalog.name.v1";
    const SCALAR_FAMILY: worth_query_decl::facade::typed::ScalarAspectType =
        worth_query_decl::facade::typed::ScalarAspectType::String;
    const SUPPORTS_ADMITTED_DECODE: bool = <String as CanonicalValue>::SUPPORTS_ADMISSION;
    fn validate(_: &CatalogName) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn canonical_contract() -> Option<ApplicationCanonicalValueContract> {
        Some(ApplicationCanonicalValueContract::of::<String>(
            ValueDecodeBounds::new(2048, 1024, 8, 8),
        ))
    }
    fn encode(value: &CatalogName) -> Result<ApplicationValue, ApplicationValueEncodeDenial> {
        encode_canonical_value(&value.0, ValueDecodeBounds::new(2048, 1024, 8, 8))
            .map(|text| ApplicationValue::String(text.into()))
            .map_err(|cause| ApplicationValueEncodeDenial::Canonical {
                binding_identity: Self::IDENTITY,
                cause,
            })
    }
}
impl ApplicationReadableScalarValueBinding for NameBinding {
    fn decode(raw: &ApplicationValue) -> Result<CatalogName, ApplicationValueDecodeDenial> {
        decode_application_canonical_value::<String>(
            raw,
            ValueDecodeBounds::new(2048, 1024, 8, 8),
            Self::IDENTITY,
        )
        .map(CatalogName)
    }
    fn decode_admitted(
        raw: &ApplicationValue,
        work: &mut ValueDecodeAdmission<'_>,
    ) -> Result<CatalogName, ApplicationValueDecodeDenial> {
        decode_application_canonical_value_with_admission::<String>(
            raw,
            ValueDecodeBounds::new(2048, 1024, 8, 8),
            Self::IDENTITY,
            work,
        )
        .map(CatalogName)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogPrincipal;
impl CatalogPrincipal {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        Catalog,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "CatalogPrincipal",
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
