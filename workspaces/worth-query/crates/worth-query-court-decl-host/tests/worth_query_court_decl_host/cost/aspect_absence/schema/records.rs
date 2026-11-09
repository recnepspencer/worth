// One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::fragment;
pub struct ClassModel;
impl fragment::Fragment for ClassModel {
    const NAME: &'static str = "class_model";
    type Bindings = ClassModelBindings;
    type Members = (Mapping, Principal, Record);
}
pub struct ClassModelBindings;
impl fragment::BindingSet for ClassModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for ClassModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for ClassModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for ClassModelBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<String> for ClassModelBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = false;
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
    type Aspects = (Identity, Classes);
    type Relations = ();
}
impl Record {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ClassModel, Self)
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
impl fragment::Record<MappingFactsValue> for ClassModelBindings {
    fn record() -> impl fragment::Shape<MappingFactsValue, Self> {
        fragment::field("ExternalId", |row: &MappingFactsValue| {
            row.external_id.clone()
        }).equality_queryable(true)
        .and("Status", |row: &MappingFactsValue| row.status).equality_queryable(true)
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
impl fragment::Record<PrincipalFactsValue> for ClassModelBindings {
    fn record() -> impl fragment::Shape<PrincipalFactsValue, Self> {
        fragment::field("PrincipalId", |row: &PrincipalFactsValue| row.principal_id).equality_queryable(true)
            .read_only()
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
impl Identity {
    pub fn reference() -> AspectSelection<ClassModel, Record, Self> {
        aspect(ClassModel, Record, Self)
    }
}
pub struct IdentityValue {
    pub key: String,
    pub sink: u64,
}
impl fragment::Record<IdentityValue> for ClassModelBindings {
    fn record() -> impl fragment::Shape<IdentityValue, Self> {
        fragment::field("Key", |row: &IdentityValue| row.key.clone()).equality_queryable(true)
            .unique()
            .and("Sink", |row: &IdentityValue| row.sink).equality_queryable(true)
            .build(|key, sink, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IdentityValue { key, sink })
            })
    }
}
pub type ExternalId =
    FieldSelection<ClassModel, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status =
    FieldSelection<ClassModel, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalId = FieldSelection<ClassModel, Principal, PrincipalFacts, u64>;
pub type Key = FieldSelection<ClassModel, Record, Identity, String>;
pub type Sink = FieldSelection<ClassModel, Record, Identity, u64>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        relation(ClassModel, Mapping, Self)
    }
}

pub type Value1 = FieldSelection<ClassModel, Record, Class1, u64>;
pub type Value3 = FieldSelection<ClassModel, Record, Class3, u64>;

impl fragment::Record<ClassValue> for ClassModelBindings {
    fn record() -> impl fragment::Shape<ClassValue, Self> {
        fragment::field("value", |row: &ClassValue| row.0)
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| Ok::<_, std::convert::Infallible>(ClassValue(value)))
    }
}
