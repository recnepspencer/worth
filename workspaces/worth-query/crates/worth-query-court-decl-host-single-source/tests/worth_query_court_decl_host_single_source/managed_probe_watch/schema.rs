//! Public schema declaring entities, aspects, and native/binary probe bindings.
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::{
    application_schema::*, authentication::*, worth_query_native_scalar_bindings,
};
pub use worth_query_testkit_document_retention::facade::managed_probe_binding::{
    TagBytes, TagBytesProbeBinding,
};

#[derive(Clone, Copy)]
pub struct CatalogSchema;
impl Fragment for CatalogSchema {
    const NAME: &'static str = "ManagedProbeWatchSchema";
    type Bindings = NoBindings;
    type Members = (super::Contribution,);
}
impl ApplicationSchema for CatalogSchema {
    const OWNER: &'static str = "worth.tests.managed_probe_watch";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchemaComposition for CatalogSchema {
    type Contributions = <Self as Fragment>::Members;
}

#[derive(Clone, Copy)]
pub struct Catalog;
impl Fragment for Catalog {
    const NAME: &'static str = "catalog";
    type Bindings = Codecs;
    type Members = (ExternalMapping, Principal, Directory, Entry);
}
worth_query_native_scalar_bindings!(pub Codecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

macro_rules! entity_decl {
    ($name:ident, $aspects:ty, $relations:ty) => {
        #[derive(Clone, Copy)]
        pub struct $name;
        impl Entity for $name {
            const NAME: &'static str = stringify!($name);
            type Aspects = $aspects;
            type Relations = $relations;
        }
    };
}
entity_decl!(ExternalMapping, (ExternalIdentity,), (MappingTarget,));
entity_decl!(Principal, (PrincipalFacts,), ());
entity_decl!(Directory, (DirectoryFacts,), (Contains,));
entity_decl!(Entry, (EntryFacts,), ());

impl Directory {
    pub fn reference() -> ApplicationEntityRef<CatalogSchema, Self> {
        entity(Catalog, Self)
    }
}
impl Entry {
    pub fn reference() -> ApplicationEntityRef<CatalogSchema, Self> {
        entity(Catalog, Self)
    }
}

pub struct ExternalIdentity;
pub struct ExternalIdentityRow {
    identity: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
impl Record<ExternalIdentityRow> for Codecs {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |r: &ExternalIdentityRow| {
            r.identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |r: &ExternalIdentityRow| r.status)
        .equality_queryable(true)
        .build(|identity, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityRow { identity, status })
        })
    }
}

pub struct PrincipalFacts;
pub struct PrincipalFactsRow {
    identity: u64,
}
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsRow;
}
impl Record<PrincipalFactsRow> for Codecs {
    fn record() -> impl Shape<PrincipalFactsRow, Self> {
        field("PrincipalIdentity", |r: &PrincipalFactsRow| r.identity)
            .equality_queryable(true)
            .read_only()
            .build(|identity, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsRow { identity })
            })
    }
}

pub struct DirectoryFacts;
pub struct DirectoryFactsRow {
    key: String,
}
impl Aspect for DirectoryFacts {
    const NAME: &'static str = "DirectoryFacts";
    type Row = DirectoryFactsRow;
}
impl Record<DirectoryFactsRow> for Codecs {
    fn record() -> impl Shape<DirectoryFactsRow, Self> {
        field("DirectoryKey", |r: &DirectoryFactsRow| r.key.clone())
            .equality_queryable(true)
            .read_only()
            .build(|key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DirectoryFactsRow { key })
            })
    }
}

pub struct EntryFacts;
pub struct EntryFactsRow {
    key: u64,
    name: String,
}
impl Aspect for EntryFacts {
    const NAME: &'static str = "EntryFacts";
    type Row = EntryFactsRow;
}
impl Record<EntryFactsRow> for Codecs {
    fn record() -> impl Shape<EntryFactsRow, Self> {
        field("EntryKey", |r: &EntryFactsRow| r.key)
            .equality_queryable(true)
            .read_only()
            .and("EntryName", |r: &EntryFactsRow| r.name.clone())
            .build(|key, name, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(EntryFactsRow { key, name })
            })
    }
}

pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<CatalogSchema, Self, ExternalMapping, Principal> {
        relation(Catalog, ExternalMapping, Self)
    }
}
pub struct Contains;
impl Relation for Contains {
    const NAME: &'static str = "Contains";
    type To = Entry;
}
impl Contains {
    pub fn reference() -> ApplicationRelationRef<CatalogSchema, Self, Directory, Entry> {
        relation(Catalog, Directory, Self)
    }
}

pub type ExternalIdField =
    FieldSelection<Catalog, ExternalMapping, ExternalIdentity, WorthQueryExternalPrincipalIdentity>;
pub type StatusField =
    FieldSelection<Catalog, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdField = FieldSelection<Catalog, Principal, PrincipalFacts, u64>;
pub type DirectoryKeyField = FieldSelection<Catalog, Directory, DirectoryFacts, String>;
pub type EntryKeyField = FieldSelection<Catalog, Entry, EntryFacts, u64>;
pub type EntryNameField = FieldSelection<Catalog, Entry, EntryFacts, String>;

pub struct PrincipalBinding;
impl PrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        CatalogSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "PrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    StatusField::at("MappingStatus"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdField::at("PrincipalIdentity"),
                ),
            },
        )
    }
}

pub struct NativePrefixIndex;
impl ApplicationManagedIndexBinding<CatalogSchema> for NativePrefixIndex {
    type SourceBinding = super::queries::SourceBinding;
    type ProbeBinding = StringApplicationValueBinding;
    type Entity = Entry;
    const NAME: &'static str = "native_prefix_index";
    const PROVIDER: &'static str = "managed-probe-watch.native-prefix.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::ExhaustiveSourceTree;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
}

pub struct BinaryTagIndex;
impl ApplicationManagedIndexBinding<CatalogSchema> for BinaryTagIndex {
    type SourceBinding = super::queries::SourceBinding;
    type ProbeBinding = TagBytesProbeBinding;
    type Entity = Entry;
    const NAME: &'static str = "binary_tag_index";
    const PROVIDER: &'static str = "managed-probe-watch.binary-tag.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::ExhaustiveSourceTree;
    const MAINTENANCE: ApplicationManagedIndexMaintenance =
        ApplicationManagedIndexMaintenance::Native {
            cold: ApplicationManagedIndexColdSource::AdmitRebuild,
        };
}
