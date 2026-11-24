//! Minimal schema records for Directory Home generated-materialization recovery courts.
use super::Catalog;
use worth_query_decl::facade::application_schema::fragment::{self, *};
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::authentication::*;

pub struct Records;
impl Fragment for Records {
    const NAME: &'static str = "records";
    type Bindings = Bindings;
    type Members = (Mapping, Principal, Artifact);
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

pub struct Artifact;
impl Entity for Artifact {
    const NAME: &'static str = "Artifact";
    type Aspects = (ArtifactFacts,);
    type Relations = ();
}
pub struct ArtifactFacts;
impl Aspect for ArtifactFacts {
    const NAME: &'static str = "ArtifactFacts";
    type Row = ArtifactRow;
}
pub struct ArtifactRow {
    payload: i64,
}
impl Record<ArtifactRow> for Bindings {
    fn record() -> impl Shape<ArtifactRow, Self> {
        field("Payload", |row: &ArtifactRow| row.payload)
            .equality_queryable(true)
            .build(|payload, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ArtifactRow { payload })
            })
    }
}

pub type External =
    FieldSelection<Records, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status =
    FieldSelection<Records, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type PrincipalValue = FieldSelection<Records, Principal, PrincipalFacts, u64>;
pub type ArtifactPayload = FieldSelection<Records, Artifact, ArtifactFacts, i64>;

pub struct Target;
impl Relation for Target {
    const NAME: &'static str = "Target";
    type To = Principal;
}
impl Target {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Mapping, Principal>
    {
        fragment::relation(Records, Mapping, Self)
    }
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        fragment::entity(Records, Self)
    }
}
impl Artifact {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        fragment::entity(Records, Self)
    }
}
