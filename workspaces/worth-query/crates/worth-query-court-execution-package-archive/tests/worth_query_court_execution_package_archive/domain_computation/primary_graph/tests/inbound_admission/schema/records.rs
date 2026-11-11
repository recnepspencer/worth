//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct InboundModel;
impl fragment::Fragment for InboundModel {
    const NAME: &'static str = "inbound_model";
    type Bindings = InboundModelBindings;
    type Members = (Mapping, Principal, Target);
}
pub struct InboundModelBindings;
impl fragment::BindingSet for InboundModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl
    fragment::Scalar<
        worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
    > for InboundModelBindings
{
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl
    fragment::Scalar<
        worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus,
    > for InboundModelBindings
{
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for InboundModelBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<String> for InboundModelBindings {
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

pub struct Target;
impl fragment::Entity for Target {
    const NAME: &'static str = "Target";
    type Aspects = (TargetFacts,);
    type Relations = ();
}
impl Target {
    pub fn reference() -> ApplicationEntityRef<InboundTestSchema, Self> {
        entity(InboundModel, Self)
    }
}
pub struct MappingFacts;
impl fragment::Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingFactsValue;
}

pub struct MappingFactsValue {
    pub mapping_identity:
        worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
    pub mapping_status:
        worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingFactsValue> for InboundModelBindings {
    fn record() -> impl fragment::Shape<MappingFactsValue, Self> {
        fragment::field("MappingIdentity", |row: &MappingFactsValue| {
            row.mapping_identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |row: &MappingFactsValue| {
            row.mapping_status
        })
        .equality_queryable(true)
        .build(
            |mapping_identity, mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingFactsValue {
                    mapping_identity,
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
impl fragment::Record<PrincipalFactsValue> for InboundModelBindings {
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
pub struct TargetFacts;
impl fragment::Aspect for TargetFacts {
    const NAME: &'static str = "TargetFacts";
    type Row = TargetFactsValue;
}

pub struct TargetFactsValue {
    pub target_key: String,
}
impl fragment::Record<TargetFactsValue> for InboundModelBindings {
    fn record() -> impl fragment::Shape<TargetFactsValue, Self> {
        fragment::field("TargetKey", |row: &TargetFactsValue| row.target_key.clone())
            .equality_queryable(true)
            .build(|target_key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(TargetFactsValue { target_key })
            })
    }
}
pub type MappingIdentity = FieldSelection<
    InboundModel,
    Mapping,
    MappingFacts,
    worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatus = FieldSelection<
    InboundModel,
    Mapping,
    MappingFacts,
    worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentity = FieldSelection<InboundModel, Principal, PrincipalFacts, u64>;
pub type TargetKey = FieldSelection<InboundModel, Target, TargetFacts, String>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<InboundTestSchema, Self, Mapping, Principal> {
        relation(InboundModel, Mapping, Self)
    }
}
