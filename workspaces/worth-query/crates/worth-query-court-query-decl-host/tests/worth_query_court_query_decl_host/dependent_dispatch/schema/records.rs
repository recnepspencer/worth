//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};
pub struct DispatchModel;
impl Fragment for DispatchModel {
    const NAME: &'static str = "dispatch_model";
    type Bindings = DispatchModelBindings;
    type Members = (Realization, Mapping, Principal, Node, ProducerCell);
}
pub struct DispatchModelBindings;
impl BindingSet for DispatchModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for DispatchModelBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl Scalar<u64> for DispatchModelBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for DispatchModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for DispatchModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
#[derive(Clone, Copy)]
pub struct Realization;
impl Entity for Realization {
    const NAME: &'static str = "Realization";
    type Aspects = (RealizationFacts,);
    type Relations = (Realizes,);
}
impl Realization {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(DispatchModel, Self)
    }
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
pub struct Node;
impl Entity for Node {
    const NAME: &'static str = "Node";
    type Aspects = (Facts, Option<OutputFacts>);
    type Relations = (GeneratedBy,);
}
impl Node {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(DispatchModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct ProducerCell;
impl Entity for ProducerCell {
    const NAME: &'static str = "ProducerCell";
    type Aspects = (CellIdentity, Option<CellComputation>);
    type Relations = ();
}
impl ProducerCell {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(DispatchModel, Self)
    }
}
pub struct RealizationFacts;
impl Aspect for RealizationFacts {
    const NAME: &'static str = "RealizationFacts";
    type Row = RealizationFactsValue;
}
impl RealizationFacts {
    pub fn reference() -> AspectSelection<DispatchModel, Realization, Self> {
        aspect(DispatchModel, Realization, Self)
    }
}
pub struct RealizationFactsValue {
    pub realization_key: String,
    pub realized_value: u64,
}
impl fragment::Record<RealizationFactsValue> for DispatchModelBindings {
    fn record() -> impl Shape<RealizationFactsValue, Self> {
        field("RealizationKey", |row: &RealizationFactsValue| {
            row.realization_key.clone()
        })
        .equality_queryable(true)
        .unique()
        .and("RealizedValue", |row: &RealizationFactsValue| {
            row.realized_value
        })
        .equality_queryable(true)
        .build(
            |realization_key, realized_value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(RealizationFactsValue {
                    realization_key,
                    realized_value,
                })
            },
        )
    }
}
pub struct MappingFacts;
impl Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingFactsValue;
}

pub struct MappingFactsValue {
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingFactsValue> for DispatchModelBindings {
    fn record() -> impl Shape<MappingFactsValue, Self> {
        field("ExternalIdentity", |row: &MappingFactsValue| {
            row.external_identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatus", |row: &MappingFactsValue| {
            row.mapping_status
        })
        .equality_queryable(true)
        .build(
            |external_identity, mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingFactsValue {
                    external_identity,
                    mapping_status,
                })
            },
        )
    }
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}

pub struct PrincipalFactsValue {
    pub principal_identity: u64,
}
impl fragment::Record<PrincipalFactsValue> for DispatchModelBindings {
    fn record() -> impl Shape<PrincipalFactsValue, Self> {
        field("PrincipalIdentity", |row: &PrincipalFactsValue| {
            row.principal_identity
        })
        .equality_queryable(true)
        .read_only()
        .build(|principal_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalFactsValue { principal_identity })
        })
    }
}
pub struct Facts;
impl Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = FactsValue;
}
impl Facts {
    pub fn reference() -> AspectSelection<DispatchModel, Node, Self> {
        aspect(DispatchModel, Node, Self)
    }
}
pub struct FactsValue {
    pub key: String,
    pub parent_name: String,
    pub value: u64,
    pub width: u64,
}
impl fragment::Record<FactsValue> for DispatchModelBindings {
    fn record() -> impl Shape<FactsValue, Self> {
        field("Key", |row: &FactsValue| row.key.clone())
            .equality_queryable(true)
            .unique()
            .and("ParentName", |row: &FactsValue| row.parent_name.clone())
            .equality_queryable(true)
            .and("Value", |row: &FactsValue| row.value)
            .equality_queryable(true)
            .and("Width", |row: &FactsValue| row.width)
            .equality_queryable(true)
            .build(
                |key, parent_name, value, width, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(FactsValue {
                        key,
                        parent_name,
                        value,
                        width,
                    })
                },
            )
    }
}
pub struct OutputFacts;
impl Aspect for OutputFacts {
    const NAME: &'static str = "OutputFacts";
    type Row = OutputFactsValue;
}

pub struct OutputFactsValue {
    pub produced: u64,
}
impl fragment::Record<OutputFactsValue> for DispatchModelBindings {
    fn record() -> impl Shape<OutputFactsValue, Self> {
        field("Produced", |row: &OutputFactsValue| row.produced)
            .equality_queryable(true)
            .build(|produced, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(OutputFactsValue { produced })
            })
    }
}
pub struct CellIdentity;
impl Aspect for CellIdentity {
    const NAME: &'static str = "CellIdentity";
    type Row = CellIdentityValue;
}

pub struct CellIdentityValue {
    pub cell_key: String,
}
impl fragment::Record<CellIdentityValue> for DispatchModelBindings {
    fn record() -> impl Shape<CellIdentityValue, Self> {
        field("CellKey", |row: &CellIdentityValue| row.cell_key.clone())
            .equality_queryable(true)
            .unique()
            .build(|cell_key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CellIdentityValue { cell_key })
            })
    }
}
pub struct CellComputation;
impl Aspect for CellComputation {
    const NAME: &'static str = "CellComputation";
    type Row = CellComputationValue;
}

pub struct CellComputationValue {
    pub cell_value: u64,
}
impl fragment::Record<CellComputationValue> for DispatchModelBindings {
    fn record() -> impl Shape<CellComputationValue, Self> {
        field("CellValue", |row: &CellComputationValue| row.cell_value)
            .equality_queryable(true)
            .build(|cell_value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CellComputationValue { cell_value })
            })
    }
}
pub type RealizationKey = FieldSelection<DispatchModel, Realization, RealizationFacts, String>;
pub type RealizedValue = FieldSelection<DispatchModel, Realization, RealizationFacts, u64>;
pub type ExternalIdentity =
    FieldSelection<DispatchModel, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type MappingStatus =
    FieldSelection<DispatchModel, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentity = FieldSelection<DispatchModel, Principal, PrincipalFacts, u64>;
pub type Key = FieldSelection<DispatchModel, Node, Facts, String>;
pub type ParentName = FieldSelection<DispatchModel, Node, Facts, String>;
pub type Value = FieldSelection<DispatchModel, Node, Facts, u64>;
pub type Width = FieldSelection<DispatchModel, Node, Facts, u64>;
pub type Produced = FieldSelection<DispatchModel, Node, OutputFacts, u64>;
pub type CellKey = FieldSelection<DispatchModel, ProducerCell, CellIdentity, String>;
pub type CellValue = FieldSelection<DispatchModel, ProducerCell, CellComputation, u64>;
pub struct Realizes;
impl Relation for Realizes {
    const NAME: &'static str = "Realizes";
    type To = Node;
}
impl Realizes {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Realization, Node> {
        relation(DispatchModel, Realization, Self)
    }
}
pub struct GeneratedBy;
impl Relation for GeneratedBy {
    const NAME: &'static str = "GeneratedBy";
    type To = Node;
}
impl GeneratedBy {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Node, Node> {
        relation(DispatchModel, Node, Self)
    }
}
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Mapping, Principal> {
        relation(DispatchModel, Mapping, Self)
    }
}
