//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct ContentModel;
impl fragment::Fragment for ContentModel {
    const NAME: &'static str = "content_model";
    type Bindings = ContentModelBindings;
    type Members = (Mapping, Principal, Record);
}
pub struct ContentModelBindings;
impl fragment::BindingSet for ContentModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for ContentModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for ContentModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for ContentModelBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<String> for ContentModelBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<ContentRefId> for ContentModelBindings {
    type Binding = ContentRefApplicationValueBinding;

    const WRITABLE: bool = true;
}
pub struct Mapping;
impl fragment::Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingFacts,);
    type Relations = (MappingTarget,);
}

pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}

pub struct Record;
impl fragment::Entity for Record {
    const NAME: &'static str = "Record";
    type Aspects = (
        Identity,
        Option<Samples>,
        Option<OtherSamples>,
        Option<VersionedSamples>,
    );
    type Relations = (ContainsSamples, ContainsOptionalSamples);
}
impl Record {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ContentModel, Self)
    }
}
pub struct MappingFacts;
impl fragment::Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingFactsValue;
}

pub struct MappingFactsValue {
    pub external_id: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingFactsValue> for ContentModelBindings {
    fn record() -> impl fragment::Shape<MappingFactsValue, Self> {
        fragment::field("ExternalId", |row: &MappingFactsValue| {
            row.external_id.clone()
        })
        .equality_queryable(true)
        .and("Status", |row: &MappingFactsValue| row.status)
        .equality_queryable(true)
        .build(|external_id, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(MappingFactsValue {
                external_id,
                status,
            })
        })
    }
}
pub struct PrincipalFacts;
impl fragment::Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}

pub struct PrincipalFactsValue {
    pub principal_id: u64,
}
impl fragment::Record<PrincipalFactsValue> for ContentModelBindings {
    fn record() -> impl fragment::Shape<PrincipalFactsValue, Self> {
        fragment::field("PrincipalId", |row: &PrincipalFactsValue| row.principal_id)
            .equality_queryable(true)
            .build(|principal_id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsValue { principal_id })
            })
    }
}
pub struct Identity;
impl fragment::Aspect for Identity {
    const NAME: &'static str = "Identity";
    type Row = IdentityValue;
}

pub struct IdentityValue {
    pub key: String,
}
impl fragment::Record<IdentityValue> for ContentModelBindings {
    fn record() -> impl fragment::Shape<IdentityValue, Self> {
        fragment::field("Key", |row: &IdentityValue| row.key.clone())
            .equality_queryable(true)
            .unique()
            .build(|key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IdentityValue { key })
            })
    }
}
pub struct Samples;
impl fragment::Aspect for Samples {
    const NAME: &'static str = "Samples";
    type Row = SamplesValue;
}
impl Samples {
    pub fn reference() -> AspectSelection<ContentModel, Record, Self> {
        aspect(ContentModel, Record, Self)
    }
}
pub struct SamplesValue {
    pub values: BoundReference<codec::GridFormat>,
    pub optional_values: Option<BoundReference<codec::GridFormat>>,
    pub unbound_values: Option<ContentRefId>,
}
impl fragment::Record<SamplesValue> for ContentModelBindings {
    fn record() -> impl fragment::Shape<SamplesValue, Self> {
        fragment::field("Values", |row: &SamplesValue| row.values)
            .and("OptionalValues", |row: &SamplesValue| row.optional_values)
            .and("UnboundValues", |row: &SamplesValue| row.unbound_values)
            .build(
                |values, optional_values, unbound_values, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(SamplesValue {
                        values,
                        optional_values,
                        unbound_values,
                    })
                },
            )
    }
}
pub struct OtherSamples;
impl fragment::Aspect for OtherSamples {
    const NAME: &'static str = "OtherSamples";
    type Row = OtherSamplesValue;
}

pub struct OtherSamplesValue {
    pub other_values: BoundReference<codec::OtherGrid>,
}
impl fragment::Record<OtherSamplesValue> for ContentModelBindings {
    fn record() -> impl fragment::Shape<OtherSamplesValue, Self> {
        fragment::field("OtherValues", |row: &OtherSamplesValue| row.other_values).build(
            |other_values, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(OtherSamplesValue { other_values })
            },
        )
    }
}
pub struct VersionedSamples;
impl fragment::Aspect for VersionedSamples {
    const NAME: &'static str = "VersionedSamples";
    type Row = VersionedSamplesValue;
}

pub struct VersionedSamplesValue {
    pub versioned_values: BoundReference<codec::GridV2>,
}
impl fragment::Record<VersionedSamplesValue> for ContentModelBindings {
    fn record() -> impl fragment::Shape<VersionedSamplesValue, Self> {
        fragment::field("VersionedValues", |row: &VersionedSamplesValue| {
            row.versioned_values
        })
        .build(|versioned_values, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(VersionedSamplesValue { versioned_values })
        })
    }
}
pub type OtherValues =
    FieldSelection<ContentModel, Record, OtherSamples, BoundReference<codec::OtherGrid>>;
pub type VersionedValues =
    FieldSelection<ContentModel, Record, VersionedSamples, BoundReference<codec::GridV2>>;
pub type ExternalId =
    FieldSelection<ContentModel, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status =
    FieldSelection<ContentModel, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalId = FieldSelection<ContentModel, Principal, PrincipalFacts, u64>;
pub type Key = FieldSelection<ContentModel, Record, Identity, String>;
pub type Values = FieldSelection<ContentModel, Record, Samples, BoundReference<codec::GridFormat>>;
pub type OptionalValues =
    FieldSelection<ContentModel, Record, Samples, BoundReference<codec::GridFormat>>;
pub type UnboundValues = FieldSelection<ContentModel, Record, Samples, ContentRefId>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        relation(ContentModel, Mapping, Self)
    }
}
pub struct ContainsSamples;
impl fragment::Relation for ContainsSamples {
    const NAME: &'static str = "ContainsSamples";
    type To = Record;
}
impl ContainsSamples {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Record, Record> {
        relation(ContentModel, Record, Self)
    }
}

impl<C: ApplicationContentBinding> fragment::Scalar<BoundReference<C>> for ContentModelBindings {
    type Binding = ReferenceBinding<C>;
    fn content_format() -> Option<ApplicationContentBindingRecipe> {
        Some(ApplicationContentBindingRecipe::of::<C>())
    }
}

pub struct ContainsOptionalSamples;
impl fragment::Relation for ContainsOptionalSamples {
    const NAME: &'static str = "ContainsOptionalSamples";
    type To = Record;
}
impl ContainsOptionalSamples {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Record, Record> {
        relation(ContentModel, Record, Self)
    }
}
