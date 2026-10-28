//! The source universe and external principal are actual installed graph meaning.
use super::{boxes, mutation, prefix, query};
use worth_query_decl::facade::{application_schema::*, authentication::*, *};

worth_query_application! { pub Catalog {
    owner: "worth.query.consumer.managed-catalog", version: (1, 0),
    contributions: [CatalogContribution],
}}
worth_query_application_contribution! { pub contribution CatalogContribution in Catalog {
    identity: "worth.query.consumer.managed-catalog.contribution.v1",
    members: |schema| {
        mutation::declare(boxes::declare(schema.entity(ExternalMapping::reference()).entity(Principal::reference())
            .entity(Directory::reference()).entity(Item::reference())
            .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
            .aspect(Principal::reference(), PrincipalFacts::reference())
            .aspect(Directory::reference(), DirectoryFacts::reference())
            .aspect(Item::reference(), ItemFacts::reference())
            .field(ExternalMapping::reference(), ExternalIdentityField::reference())
            .field(ExternalMapping::reference(), MappingStatusField::reference())
            .field(Principal::reference(), PrincipalIdentityField::reference())
            .field(Directory::reference(), DirectoryIdentity::reference())
            .field(Item::reference(), ItemIdentity::reference())
            .field(Item::reference(), ItemName::reference())
            .field(Item::reference(), ItemEnabled::reference())
            .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
            .relation(Contains::reference(), Directory::reference(), Item::reference())
            .principal_binding(CatalogPrincipal::reference())
            .application_query(query::source_definition())
            .application_query_binding::<query::SourceBinding>()
            .application_query(query::prefix_definition())
            .application_query_binding::<query::PrefixBinding>()
            .managed_index(ApplicationManagedIndexDefinition::<Catalog, prefix::PrefixIndex>::new())))
    }
}}
use worth_query_decl::facade::application_query::ApplicationManagedIndexDefinition;
worth_query_entity!(pub ExternalMapping for Catalog);
worth_query_entity!(pub Principal for Catalog);
worth_query_entity!(pub Directory for Catalog);
worth_query_entity!(pub Item for Catalog);
worth_query_aspect!(pub ExternalIdentity for Catalog, ExternalMapping; identity = AspectIdentity(0x919001), revision = AspectContractRevision(1));
worth_query_aspect!(pub PrincipalFacts for Catalog, Principal; identity = AspectIdentity(0x919002), revision = AspectContractRevision(1));
worth_query_aspect!(pub DirectoryFacts for Catalog, Directory; identity = AspectIdentity(0x919003), revision = AspectContractRevision(1));
worth_query_aspect!(pub ItemFacts for Catalog, Item; identity = AspectIdentity(0x919004), revision = AspectContractRevision(1));
worth_query_field!(pub ExternalIdentityField for Catalog, ExternalMapping, ExternalIdentity: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub MappingStatusField for Catalog, ExternalMapping, ExternalIdentity: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub PrincipalIdentityField for Catalog, Principal, PrincipalFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub DirectoryIdentity for Catalog, Directory, DirectoryFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub ItemIdentity for Catalog, Item, ItemFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_value_binding!(pub NameBinding for String {
    identity: "managed-catalog.name.v1", scalar: String,
    canonical: ValueDecodeBounds::new(2048, 1024, 8, 8),
});
worth_query_field!(pub ItemName for Catalog, Item, ItemFacts: String => NameBinding, read_write, equality);
worth_query_field!(pub ItemEnabled for Catalog, Item, ItemFacts: bool => BoolApplicationValueBinding, read_only, equality);
worth_query_relation!(pub MappingTarget in Catalog, ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_relation!(pub Contains in Catalog, Directory => Item; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(pub CatalogPrincipal in Catalog, mapping ExternalMapping {
    identity: ExternalIdentityField, status: MappingStatusField,
    target: MappingTarget => Principal, principal_identity: PrincipalIdentityField
});
