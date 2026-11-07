//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct AggregateSchemaRecords;
impl fragment::Fragment for AggregateSchemaRecords {
    const NAME: &'static str = "aggregate_schema_records";
    type Bindings = AggregateSchemaRecordsBindings;
    type Members = (
        AggregateExternalMapping,
        AggregatePrincipal,
        AggregateSource,
        AggregateTarget,
    );
}
pub struct AggregateSchemaRecordsBindings;
impl fragment::BindingSet for AggregateSchemaRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for AggregateSchemaRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for AggregateSchemaRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for AggregateSchemaRecordsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<String> for AggregateSchemaRecordsBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<i64> for AggregateSchemaRecordsBindings {
    type Binding = I64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
pub struct AggregateExternalMapping;
impl fragment::Entity for AggregateExternalMapping {
    const NAME: &'static str = "AggregateExternalMapping";
    type Aspects = (AggregateExternalIdentity,);
    type Relations = (AggregateMappingTarget,);
}

pub struct AggregatePrincipal;
impl fragment::Entity for AggregatePrincipal {
    const NAME: &'static str = "AggregatePrincipal";
    type Aspects = (AggregatePrincipalIdentity,);
    type Relations = ();
}

pub struct AggregateSource;
impl fragment::Entity for AggregateSource {
    const NAME: &'static str = "AggregateSource";
    type Aspects = (SourceFacts,);
    type Relations = (AggregateContribution,);
}
impl AggregateSource {
    pub fn reference() -> ApplicationEntityRef<AggregateSchema, Self> {
        entity(AggregateSchemaRecords, Self)
    }
}
pub struct AggregateTarget;
impl fragment::Entity for AggregateTarget {
    const NAME: &'static str = "AggregateTarget";
    type Aspects = (TargetFacts,);
    type Relations = ();
}
impl AggregateTarget {
    pub fn reference() -> ApplicationEntityRef<AggregateSchema, Self> {
        entity(AggregateSchemaRecords, Self)
    }
}
pub struct AggregateExternalIdentity;
impl fragment::Aspect for AggregateExternalIdentity {
    const NAME: &'static str = "AggregateExternalIdentity";
    type Row = AggregateExternalIdentityValue;
}

pub struct AggregateExternalIdentityValue {
    pub aggregate_external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub aggregate_mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<AggregateExternalIdentityValue> for AggregateSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<AggregateExternalIdentityValue, Self> {
        fragment::field(
            "AggregateExternalIdentityField",
            |row: &AggregateExternalIdentityValue| row.aggregate_external_identity_field.clone(),
        )
        .equality_queryable(true)
        .and(
            "AggregateMappingStatusField",
            |row: &AggregateExternalIdentityValue| row.aggregate_mapping_status_field,
        )
        .equality_queryable(true)
        .build(
            |aggregate_external_identity_field,
             aggregate_mapping_status_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AggregateExternalIdentityValue {
                    aggregate_external_identity_field,
                    aggregate_mapping_status_field,
                })
            },
        )
    }
}
pub struct AggregatePrincipalIdentity;
impl fragment::Aspect for AggregatePrincipalIdentity {
    const NAME: &'static str = "AggregatePrincipalIdentity";
    type Row = AggregatePrincipalIdentityValue;
}

pub struct AggregatePrincipalIdentityValue {
    pub aggregate_principal_identity_field: u64,
}
impl fragment::Record<AggregatePrincipalIdentityValue> for AggregateSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<AggregatePrincipalIdentityValue, Self> {
        fragment::field(
            "AggregatePrincipalIdentityField",
            |row: &AggregatePrincipalIdentityValue| row.aggregate_principal_identity_field,
        )
        .equality_queryable(true)
        .build(
            |aggregate_principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AggregatePrincipalIdentityValue {
                    aggregate_principal_identity_field,
                })
            },
        )
    }
}
pub struct SourceFacts;
impl fragment::Aspect for SourceFacts {
    const NAME: &'static str = "SourceFacts";
    type Row = SourceFactsValue;
}

pub struct SourceFactsValue {
    pub source_identity: String,
    pub source_amount: Option<i64>,
}
impl fragment::Record<SourceFactsValue> for AggregateSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<SourceFactsValue, Self> {
        fragment::field("SourceIdentity", |row: &SourceFactsValue| {
            row.source_identity.clone()
        })
        .equality_queryable(true)
        .and("SourceAmount", |row: &SourceFactsValue| row.source_amount)
        .build(
            |source_identity, source_amount, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SourceFactsValue {
                    source_identity,
                    source_amount,
                })
            },
        )
    }
}
pub struct TargetFacts;
impl fragment::Aspect for TargetFacts {
    const NAME: &'static str = "TargetFacts";
    type Row = TargetFactsValue;
}

pub struct TargetFactsValue {
    pub target_identity: String,
}
impl fragment::Record<TargetFactsValue> for AggregateSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<TargetFactsValue, Self> {
        fragment::field("TargetIdentity", |row: &TargetFactsValue| {
            row.target_identity.clone()
        })
        .equality_queryable(true)
        .build(|target_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(TargetFactsValue { target_identity })
        })
    }
}
pub type AggregateExternalIdentityField = FieldSelection<
    AggregateSchemaRecords,
    AggregateExternalMapping,
    AggregateExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type AggregateMappingStatusField = FieldSelection<
    AggregateSchemaRecords,
    AggregateExternalMapping,
    AggregateExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type AggregatePrincipalIdentityField =
    FieldSelection<AggregateSchemaRecords, AggregatePrincipal, AggregatePrincipalIdentity, u64>;
pub type SourceIdentity =
    FieldSelection<AggregateSchemaRecords, AggregateSource, SourceFacts, String>;
pub type SourceAmount = FieldSelection<AggregateSchemaRecords, AggregateSource, SourceFacts, i64>;
pub type TargetIdentity =
    FieldSelection<AggregateSchemaRecords, AggregateTarget, TargetFacts, String>;
pub struct AggregateMappingTarget;
impl fragment::Relation for AggregateMappingTarget {
    const NAME: &'static str = "AggregateMappingTarget";
    type To = AggregatePrincipal;
}
impl AggregateMappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<AggregateSchema, Self, AggregateExternalMapping, AggregatePrincipal>
    {
        relation(AggregateSchemaRecords, AggregateExternalMapping, Self)
    }
}
pub struct AggregateContribution;
impl fragment::Relation for AggregateContribution {
    const NAME: &'static str = "AggregateContribution";
    type To = AggregateTarget;
}
impl AggregateContribution {
    pub fn reference(
    ) -> ApplicationRelationRef<AggregateSchema, Self, AggregateSource, AggregateTarget> {
        relation(AggregateSchemaRecords, AggregateSource, Self)
    }
}
