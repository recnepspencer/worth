//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::fragment;
pub struct OwnershipModel<const CONTENT: bool = true>;
impl fragment::Fragment for OwnershipModel<true> {
    const NAME: &'static str = "ownership_model";
    type Bindings = OwnershipModelBindings;
    type Members = (Mapping, NativeNode, heterogeneous::Body);
}
impl fragment::Fragment for OwnershipModel<false> {
    const NAME: &'static str = "ownership_model";
    type Bindings = OwnershipModelBindings;
    type Members = (Mapping, NativeNode<false>, heterogeneous::Body);
}
pub struct OwnershipModelBindings;
impl fragment::BindingSet for OwnershipModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for OwnershipModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for OwnershipModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for OwnershipModelBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
#[derive(Clone, Copy)]
pub struct Mapping;
impl fragment::Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingFacts,);
    type Relations = (MapsTo,);
}
impl Mapping {
    pub fn reference<S: super::Scenario>() -> ApplicationEntityRef<S, Self> {
        entity(OwnershipModel::<true>, Self)
    }
}
#[derive(Clone, Copy)]
pub struct NativeNode<const CONTENT: bool = true>;
impl fragment::Entity for NativeNode<true> {
    const NAME: &'static str = "Node";
    type Aspects = (Facts, Option<Values>, Option<content::Payload>);
    type Relations = (
        Peer,
        stage_schema::PreviousStage,
        stage_schema::StageVisible,
        stage_schema::OwnCell,
        stage_schema::SourceLink,
    );
}
impl fragment::Entity for NativeNode<false> {
    const NAME: &'static str = "Node";
    type Aspects = (Facts, Option<Values>);
    type Relations = (
        Peer,
        stage_schema::PreviousStage,
        stage_schema::StageVisible,
        stage_schema::OwnCell,
        stage_schema::SourceLink,
    );
}
impl NativeNode {
    pub fn reference<S: super::Scenario>() -> ApplicationEntityRef<S, Self> {
        entity(OwnershipModel::<true>, Self)
    }
}
pub struct MappingFacts;
impl fragment::Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingFactsValue;
}
impl MappingFacts {
    pub fn reference() -> AspectSelection<OwnershipModel, Mapping, Self> {
        aspect(OwnershipModel::<true>, Mapping, Self)
    }
}
pub struct MappingFactsValue {
    pub external_id: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingFactsValue> for OwnershipModelBindings {
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
pub struct Facts;
impl fragment::Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = FactsValue;
}
impl Facts {
    pub fn reference() -> AspectSelection<OwnershipModel, NativeNode, Self> {
        aspect(OwnershipModel::<true>, NativeNode::<true>, Self)
    }
}
pub struct FactsValue {
    pub id: u64,
}
impl fragment::Record<FactsValue> for OwnershipModelBindings {
    fn record() -> impl fragment::Shape<FactsValue, Self> {
        fragment::field("Id", |row: &FactsValue| row.id)
            .read_only()
            .unique()
            .build(|id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(FactsValue { id })
            })
    }
}
pub struct Values;
impl fragment::Aspect for Values {
    const NAME: &'static str = "Values";
    type Row = ValuesValue;
}
impl Values {
    pub fn reference() -> AspectSelection<OwnershipModel, NativeNode, Self> {
        aspect(OwnershipModel::<true>, NativeNode::<true>, Self)
    }
}
pub struct ValuesValue {
    pub value: u64,
}
impl fragment::Record<ValuesValue> for OwnershipModelBindings {
    fn record() -> impl fragment::Shape<ValuesValue, Self> {
        fragment::field("Value", |row: &ValuesValue| row.value).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ValuesValue { value })
            },
        )
    }
}
pub type ExternalId =
    FieldSelection<OwnershipModel, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status =
    FieldSelection<OwnershipModel, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type RawId<const CONTENT: bool> =
    FieldSelection<OwnershipModel<CONTENT>, NativeNode<CONTENT>, Facts, u64>;
pub type RawValue<const CONTENT: bool> =
    FieldSelection<OwnershipModel<CONTENT>, NativeNode<CONTENT>, Values, u64>;
