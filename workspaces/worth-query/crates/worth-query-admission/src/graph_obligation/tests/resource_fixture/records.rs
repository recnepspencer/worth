//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct ResourceModel;
impl fragment::Fragment for ResourceModel {
    const NAME: &'static str = "resource_model";
    type Bindings = ResourceModelBindings;
    type Members = (Mapping, Principal);
}
pub struct ResourceModelBindings;
impl fragment::BindingSet for ResourceModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for ResourceModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for ResourceModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for ResourceModelBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
pub struct Mapping;
impl fragment::Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingFacts,);
    type Relations = (Target,);
}

pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(ResourceModel, Self)
    }
}
pub struct MappingFacts;
impl fragment::Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingFactsValue;
}

pub struct MappingFactsValue {
    pub mapping_identity: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingFactsValue> for ResourceModelBindings {
    fn record() -> impl fragment::Shape<MappingFactsValue, Self> {
        fragment::field("MappingIdentity", |row: &MappingFactsValue| {
            row.mapping_identity.clone()
        })
        .and("MappingStatus", |row: &MappingFactsValue| {
            row.mapping_status
        })
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
impl fragment::Record<PrincipalFactsValue> for ResourceModelBindings {
    fn record() -> impl fragment::Shape<PrincipalFactsValue, Self> {
        fragment::field("PrincipalIdentity", |row: &PrincipalFactsValue| {
            row.principal_identity
        })
        .build(|principal_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalFactsValue { principal_identity })
        })
    }
}
pub type MappingIdentity =
    FieldSelection<ResourceModel, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatus =
    FieldSelection<ResourceModel, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentity = FieldSelection<ResourceModel, Principal, PrincipalFacts, u64>;
pub struct Target;
impl fragment::Relation for Target {
    const NAME: &'static str = "Target";
    type To = Principal;
}
impl Target {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        relation(ResourceModel, Mapping, Self)
    }
}
