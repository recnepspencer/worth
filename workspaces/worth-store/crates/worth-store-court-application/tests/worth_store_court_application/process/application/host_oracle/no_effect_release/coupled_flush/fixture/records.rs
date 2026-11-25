//! Minimal 2-coupled-member schema records for original-write flush interruption.
use super::Catalog;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;

pub struct Records;
impl Fragment for Records {
    const NAME: &'static str = "coupled_records";
    type Bindings = Bindings;
    type Members = (Mapping, Principal, Model, ArtifactA, ArtifactB);
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for Bindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<i64> for Bindings {
    type Binding = I64ApplicationValueBinding;
    const WRITABLE: bool = true;
}

pub struct Mapping;
impl Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingFacts,);
    type Relations = (Target,);
}
pub struct MappingFacts;
impl Aspect for MappingFacts {
    const NAME: &'static str = "MappingFacts";
    type Row = MappingRow;
}
pub struct MappingRow {
    external: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
impl Record<MappingRow> for Bindings {
    fn record() -> impl Shape<MappingRow, Self> {
        field("External", |row: &MappingRow| row.external.clone())
            .equality_queryable(true)
            .and("Status", |row: &MappingRow| row.status)
            .equality_queryable(true)
            .build(|external, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingRow { external, status })
            })
    }
}

pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalRow;
}
pub struct PrincipalRow {
    value: u64,
}
impl Record<PrincipalRow> for Bindings {
    fn record() -> impl Shape<PrincipalRow, Self> {
        field("Value", |row: &PrincipalRow| row.value)
            .equality_queryable(true)
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow { value })
            })
    }
}

pub struct Model;
impl Entity for Model {
    const NAME: &'static str = "Model";
    type Aspects = (ModelFacts,);
    type Relations = ();
}
pub struct ModelFacts;
impl Aspect for ModelFacts {
    const NAME: &'static str = "ModelFacts";
    type Row = ModelRow;
}
pub struct ModelRow {
    key: u64,
    data: i64,
}
impl Record<ModelRow> for Bindings {
    fn record() -> impl Shape<ModelRow, Self> {
        field("Key", |row: &ModelRow| row.key)
            .equality_queryable(true)
            .and("Data", |row: &ModelRow| row.data)
            .equality_queryable(true)
            .build(|key, data, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ModelRow { key, data })
            })
    }
}

pub struct ArtifactA;
impl Entity for ArtifactA {
    const NAME: &'static str = "ArtifactA";
    type Aspects = (ArtifactAFacts,);
    type Relations = ();
}
pub struct ArtifactAFacts;
impl Aspect for ArtifactAFacts {
    const NAME: &'static str = "ArtifactAFacts";
    type Row = ArtifactARow;
}
pub struct ArtifactARow {
    payload: i64,
}
impl Record<ArtifactARow> for Bindings {
    fn record() -> impl Shape<ArtifactARow, Self> {
        field("Payload", |row: &ArtifactARow| row.payload)
            .equality_queryable(true)
            .build(|payload, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ArtifactARow { payload })
            })
    }
}

pub struct ArtifactB;
impl Entity for ArtifactB {
    const NAME: &'static str = "ArtifactB";
    type Aspects = (ArtifactBFacts,);
    type Relations = ();
}
pub struct ArtifactBFacts;
impl Aspect for ArtifactBFacts {
    const NAME: &'static str = "ArtifactBFacts";
    type Row = ArtifactBRow;
}
pub struct ArtifactBRow {
    payload: i64,
}
impl Record<ArtifactBRow> for Bindings {
    fn record() -> impl Shape<ArtifactBRow, Self> {
        field("Payload", |row: &ArtifactBRow| row.payload)
            .equality_queryable(true)
            .build(|payload, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ArtifactBRow { payload })
            })
    }
}

pub type External =
    FieldSelection<Records, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status =
    FieldSelection<Records, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalValue = FieldSelection<Records, Principal, PrincipalFacts, u64>;
pub type ModelKey = FieldSelection<Records, Model, ModelFacts, u64>;
pub type ModelData = FieldSelection<Records, Model, ModelFacts, i64>;
pub type PayloadA = FieldSelection<Records, ArtifactA, ArtifactAFacts, i64>;
pub type PayloadB = FieldSelection<Records, ArtifactB, ArtifactBFacts, i64>;

pub struct Target;
impl Relation for Target {
    const NAME: &'static str = "Target";
    type To = Principal;
}
impl Target {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Mapping, Principal> {
        fragment::relation(Records, Mapping, Self)
    }
}
impl Model {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        fragment::entity(Records, Self)
    }
}
impl ArtifactA {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        fragment::entity(Records, Self)
    }
}
impl ArtifactB {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        fragment::entity(Records, Self)
    }
}
