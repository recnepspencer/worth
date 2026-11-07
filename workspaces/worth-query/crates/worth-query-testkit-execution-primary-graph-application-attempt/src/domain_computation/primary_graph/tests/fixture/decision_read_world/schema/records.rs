//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct DecisionRecords;
impl fragment::Fragment for DecisionRecords {
    const NAME: &'static str = "decision_records";
    type Bindings = DecisionRecordsBindings;
    type Members = (Mapping, Principal);
}
pub struct DecisionRecordsBindings;
impl fragment::BindingSet for DecisionRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for DecisionRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for DecisionRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for DecisionRecordsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
pub struct Mapping;
impl fragment::Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}

pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<DecisionReadSchema, Self> {
        entity(DecisionRecords, Self)
    }
}
pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}

pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for DecisionRecordsBindings {
    fn record() -> impl fragment::Shape<ExternalIdentityValue, Self> {
        fragment::field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |row: &ExternalIdentityValue| {
            row.mapping_status
        })
        .equality_queryable(true)
        .build(
            |external_identity_field, mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status,
                })
            },
        )
    }
}
pub struct PrincipalFacts;
impl fragment::Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}

pub struct PrincipalFactsValue {
    pub principal_identity: u64,
}
impl fragment::Record<PrincipalFactsValue> for DecisionRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalFactsValue, Self> {
        fragment::field("PrincipalIdentity", |row: &PrincipalFactsValue| {
            row.principal_identity
        })
        .equality_queryable(true)
        .build(|principal_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalFactsValue { principal_identity })
        })
    }
}
pub type ExternalIdentityField =
    FieldSelection<DecisionRecords, Mapping, ExternalIdentity, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatus =
    FieldSelection<DecisionRecords, Mapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentity = FieldSelection<DecisionRecords, Principal, PrincipalFacts, u64>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<DecisionReadSchema, Self, Mapping, Principal> {
        relation(DecisionRecords, Mapping, Self)
    }
}
