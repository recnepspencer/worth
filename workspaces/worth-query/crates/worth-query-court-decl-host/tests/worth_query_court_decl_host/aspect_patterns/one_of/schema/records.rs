//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct PatternModel;
impl fragment::Fragment for PatternModel {
    const NAME: &'static str = "pattern_model";
    type Bindings = PatternModelBindings;
    type Members = (Mapping, Principal, Record);
}
pub struct PatternModelBindings;
impl fragment::BindingSet for PatternModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for PatternModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for PatternModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for PatternModelBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<String> for PatternModelBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
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
    type Aspects = (Identity, Exclusive<Meaning>);
    type Relations = ();
}
impl Record {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(PatternModel, Self)
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
impl fragment::Record<MappingFactsValue> for PatternModelBindings {
    fn record() -> impl fragment::Shape<MappingFactsValue, Self> {
        fragment::field("ExternalId", |row: &MappingFactsValue| {
            row.external_id.clone()
        })
        .and("Status", |row: &MappingFactsValue| row.status)
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
impl fragment::Record<PrincipalFactsValue> for PatternModelBindings {
    fn record() -> impl fragment::Shape<PrincipalFactsValue, Self> {
        fragment::field("PrincipalId", |row: &PrincipalFactsValue| row.principal_id)
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

pub struct IdentityValue {
    pub key: String,
}
impl fragment::Record<IdentityValue> for PatternModelBindings {
    fn record() -> impl fragment::Shape<IdentityValue, Self> {
        fragment::field("Key", |row: &IdentityValue| row.key.clone())
            .unique()
            .build(|key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(IdentityValue { key })
            })
    }
}
pub struct First;
impl fragment::Aspect for First {
    const NAME: &'static str = "First";
    type Row = FirstRow;
}
impl First {
    pub fn reference() -> AspectSelection<PatternModel, Record, Self> {
        aspect(PatternModel, Record, Self)
    }
}
#[derive(Clone, Copy)]
pub struct FirstRow {
    pub first_value: u64,
}
impl fragment::Record<FirstRow> for PatternModelBindings {
    fn record() -> impl fragment::Shape<FirstRow, Self> {
        fragment::field("FirstValue", |row: &FirstRow| row.first_value).build(
            |first_value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(FirstRow { first_value })
            },
        )
    }
}
pub struct Second;
impl fragment::Aspect for Second {
    const NAME: &'static str = "Second";
    type Row = SecondRow;
}
impl Second {
    pub fn reference() -> AspectSelection<PatternModel, Record, Self> {
        aspect(PatternModel, Record, Self)
    }
}
#[derive(Clone, Copy)]
pub struct SecondRow {
    pub second_value: u64,
    pub second_required: u64,
}
impl fragment::Record<SecondRow> for PatternModelBindings {
    fn record() -> impl fragment::Shape<SecondRow, Self> {
        fragment::field("SecondValue", |row: &SecondRow| row.second_value)
            .and("SecondRequired", |row: &SecondRow| row.second_required)
            .build(
                |second_value, second_required, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(SecondRow {
                        second_value,
                        second_required,
                    })
                },
            )
    }
}
pub struct Third;
impl fragment::Aspect for Third {
    const NAME: &'static str = "Third";
    type Row = ThirdRow;
}
impl Third {
    pub fn reference() -> AspectSelection<PatternModel, Record, Self> {
        aspect(PatternModel, Record, Self)
    }
}
#[derive(Clone, Copy)]
pub struct ThirdRow {
    pub third_value: u64,
}
impl fragment::Record<ThirdRow> for PatternModelBindings {
    fn record() -> impl fragment::Shape<ThirdRow, Self> {
        fragment::field("ThirdValue", |row: &ThirdRow| row.third_value).build(
            |third_value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ThirdRow { third_value })
            },
        )
    }
}
pub type ExternalId =
    FieldSelection<PatternModel, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status =
    FieldSelection<PatternModel, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalId = FieldSelection<PatternModel, Principal, PrincipalFacts, u64>;
pub type Key = FieldSelection<PatternModel, Record, Identity, String>;
pub type FirstValue = FieldSelection<PatternModel, Record, First, u64>;
pub type SecondValue = FieldSelection<PatternModel, Record, Second, u64>;
pub type SecondRequired = FieldSelection<PatternModel, Record, Second, u64>;
pub type ThirdValue = FieldSelection<PatternModel, Record, Third, u64>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        relation(PatternModel, Mapping, Self)
    }
}

pub struct Meaning;
impl fragment::OneOf for Meaning {
    type Row = MeaningRow;
    type Cases = (First, Second, Third);
}
#[expect(
    dead_code,
    reason = "This fixture projects case fields separately and consumes the enum discriminant."
)]
pub enum MeaningRow {
    First(FirstRow),
    Second(SecondRow),
    Third(ThirdRow),
}
impl From<FirstRow> for MeaningRow {
    fn from(value: FirstRow) -> Self {
        Self::First(value)
    }
}
impl From<SecondRow> for MeaningRow {
    fn from(value: SecondRow) -> Self {
        Self::Second(value)
    }
}
impl From<ThirdRow> for MeaningRow {
    fn from(value: ThirdRow) -> Self {
        Self::Third(value)
    }
}
impl MeaningRow {
    pub fn class(&self) -> u64 {
        match self {
            Self::First(_) => 1,
            Self::Second(_) => 2,
            Self::Third(_) => 3,
        }
    }
}
