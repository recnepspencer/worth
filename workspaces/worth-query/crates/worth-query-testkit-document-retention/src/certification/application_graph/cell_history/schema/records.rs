//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{fragment::*, *};
#[derive(Clone, Copy)]
pub struct CellModel;
impl Fragment for CellModel {
    const NAME: &'static str = "cell_model";
    type Bindings = CellModelBindings;
    type Members = (ExternalMapping, Principal, Feature, Cell);
}
pub struct CellModelBindings;
impl BindingSet for CellModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for CellModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for CellModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl Scalar<u64> for CellModelBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
#[derive(Clone, Copy)]
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(CellModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(CellModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Feature;
impl Entity for Feature {
    const NAME: &'static str = "Feature";
    type Aspects = (FeatureFacts,);
    type Relations = (OwnsCell,);
}
impl Feature {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(CellModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Cell;
impl Entity for Cell {
    const NAME: &'static str = "Cell";
    type Aspects = (CellFacts,);
    type Relations = ();
}
impl Cell {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(CellModel, Self)
    }
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<CellModel, ExternalMapping, Self> {
        aspect(CellModel, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for CellModelBindings {
    fn record() -> impl Shape<ExternalIdentityValue, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityValue| {
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
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
impl PrincipalFacts {
    pub fn reference() -> AspectSelection<CellModel, Principal, Self> {
        aspect(CellModel, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity: u64,
}
impl Record<PrincipalFactsValue> for CellModelBindings {
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
pub struct FeatureFacts;
impl Aspect for FeatureFacts {
    const NAME: &'static str = "FeatureFacts";
    type Row = FeatureFactsValue;
}
impl FeatureFacts {
    pub fn reference() -> AspectSelection<CellModel, Feature, Self> {
        aspect(CellModel, Feature, Self)
    }
}
pub struct FeatureFactsValue {
    pub feature_key: u64,
}
impl Record<FeatureFactsValue> for CellModelBindings {
    fn record() -> impl Shape<FeatureFactsValue, Self> {
        field("FeatureKey", |row: &FeatureFactsValue| row.feature_key)
            .equality_queryable(true)
            .read_only()
            .build(|feature_key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(FeatureFactsValue { feature_key })
            })
    }
}
pub struct CellFacts;
impl Aspect for CellFacts {
    const NAME: &'static str = "CellFacts";
    type Row = CellFactsValue;
}
impl CellFacts {
    pub fn reference() -> AspectSelection<CellModel, Cell, Self> {
        aspect(CellModel, Cell, Self)
    }
}
pub struct CellFactsValue {
    pub cell_key: u64,
    pub cell_value: u64,
}
impl Record<CellFactsValue> for CellModelBindings {
    fn record() -> impl Shape<CellFactsValue, Self> {
        field("CellKey", |row: &CellFactsValue| row.cell_key)
            .equality_queryable(true)
            .read_only()
            .and("CellValue", |row: &CellFactsValue| row.cell_value)
            .equality_queryable(true)
            .build(|cell_key, cell_value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(CellFactsValue {
                    cell_key,
                    cell_value,
                })
            })
    }
}
pub type ExternalIdentityField = FieldSelection<
    CellModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatus =
    FieldSelection<CellModel, ExternalMapping, ExternalIdentity, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentity = FieldSelection<CellModel, Principal, PrincipalFacts, u64>;
pub type FeatureKey = FieldSelection<CellModel, Feature, FeatureFacts, u64>;
pub type CellKey = FieldSelection<CellModel, Cell, CellFacts, u64>;
pub type CellValue = FieldSelection<CellModel, Cell, CellFacts, u64>;
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, ExternalMapping, Principal> {
        relation(CellModel, ExternalMapping, Self)
    }
}
pub struct OwnsCell;
impl Relation for OwnsCell {
    const NAME: &'static str = "OwnsCell";
    type To = Cell;
}
impl OwnsCell {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Feature, Cell> {
        relation(CellModel, Feature, Self)
    }
}
