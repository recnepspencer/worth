//! One record model owns field lowering and reusable scalar bindings.
use super::*;
pub struct SourceModel;
impl Fragment for SourceModel {
    const NAME: &'static str = "source_model";
    type Bindings = SourceModelBindings;
    type Members = (
        Mapping,
        Principal,
        Source,
        Generated,
        roster::Member,
        roster::Note,
    );
}
pub struct SourceFeature;
impl Fragment for SourceFeature {
    const NAME: &'static str = "source_feature";
    type Bindings = SourceModelBindings;
    type Members = (
        Extend<Source, (Option<SourceFacts>,)>,
        Extend<roster::Note, (Option<roster::NoteMark>,)>,
    );
}
pub struct SourceModelBindings;
impl BindingSet for SourceModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for SourceModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for SourceModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl Scalar<u64> for SourceModelBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl Scalar<String> for SourceModelBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = false;
}
#[derive(Clone, Copy)]
pub struct Mapping;
impl Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingFacts,);
    type Relations = (MappingTarget,);
}

#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}

#[derive(Clone, Copy)]
pub struct Source;
impl Entity for Source {
    const NAME: &'static str = "Source";
    type Aspects = ();
    type Relations = (SourceMember, roster::Head);
}
impl Source {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(SourceModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Generated;
impl Entity for Generated {
    const NAME: &'static str = "Generated";
    type Aspects = (GeneratedFacts,);
    type Relations = (GeneratedMember,);
}
impl Generated {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(SourceModel, Self)
    }
}
pub struct MappingFacts;
impl Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingFactsValue;
}

pub struct MappingFactsValue {
    pub external_id: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl Record<MappingFactsValue> for SourceModelBindings {
    fn record() -> impl Shape<MappingFactsValue, Self> {
        field("ExternalId", |row: &MappingFactsValue| {
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
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}

pub struct PrincipalFactsValue {
    pub principal_id: u64,
}
impl Record<PrincipalFactsValue> for SourceModelBindings {
    fn record() -> impl Shape<PrincipalFactsValue, Self> {
        field("PrincipalId", |row: &PrincipalFactsValue| row.principal_id)
            .equality_queryable(true)
            .read_only()
            .build(|principal_id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsValue { principal_id })
            })
    }
}
pub struct SourceFacts;
impl Aspect for SourceFacts {
    const NAME: &'static str = "SourceFacts";
    type Row = SourceFactsValue;
}

pub struct SourceFactsValue {
    pub source_key: String,
    pub source_value: u64,
}
impl Record<SourceFactsValue> for SourceModelBindings {
    fn record() -> impl Shape<SourceFactsValue, Self> {
        field("SourceKey", |row: &SourceFactsValue| row.source_key.clone())
            .equality_queryable(true)
            .unique()
            .and("SourceValue", |row: &SourceFactsValue| row.source_value)
            .equality_queryable(true)
            .build(
                |source_key, source_value, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(SourceFactsValue {
                        source_key,
                        source_value,
                    })
                },
            )
    }
}
pub struct GeneratedFacts;
impl Aspect for GeneratedFacts {
    const NAME: &'static str = "GeneratedFacts";
    type Row = GeneratedFactsValue;
}

pub struct GeneratedFactsValue {
    pub generated_key: String,
    pub generated_value: u64,
}
impl Record<GeneratedFactsValue> for SourceModelBindings {
    fn record() -> impl Shape<GeneratedFactsValue, Self> {
        field("GeneratedKey", |row: &GeneratedFactsValue| {
            row.generated_key.clone()
        })
        .equality_queryable(true)
        .unique()
        .and("GeneratedValue", |row: &GeneratedFactsValue| {
            row.generated_value
        })
        .equality_queryable(true)
        .build(
            |generated_key, generated_value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(GeneratedFactsValue {
                    generated_key,
                    generated_value,
                })
            },
        )
    }
}
pub type ExternalId =
    FieldSelection<SourceModel, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status =
    FieldSelection<SourceModel, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalId = FieldSelection<SourceModel, Principal, PrincipalFacts, u64>;
pub type SourceKey = FieldSelection<SourceFeature, Source, SourceFacts, String>;
pub type SourceValue = FieldSelection<SourceFeature, Source, SourceFacts, u64>;
pub type GeneratedKey = FieldSelection<SourceModel, Generated, GeneratedFacts, String>;
pub type GeneratedValue = FieldSelection<SourceModel, Generated, GeneratedFacts, u64>;
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        relation(SourceModel, Mapping, Self)
    }
}
pub struct SourceMember;
impl Relation for SourceMember {
    const NAME: &'static str = "SourceMember";
    type To = Source;
}
impl SourceMember {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Source, Source> {
        relation(SourceModel, Source, Self)
    }
}
pub struct GeneratedMember;
impl Relation for GeneratedMember {
    const NAME: &'static str = "GeneratedMember";
    type To = Generated;
}
impl GeneratedMember {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Generated, Generated> {
        relation(SourceModel, Generated, Self)
    }
}
