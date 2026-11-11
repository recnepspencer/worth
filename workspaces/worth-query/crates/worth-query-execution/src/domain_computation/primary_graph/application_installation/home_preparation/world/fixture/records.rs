//! Record membership owns schema fields and their original scalar contracts.
use super::*;
use worth_query_declaration::facade::application_schema::fragment;
pub struct IdentityRecords;
impl Fragment for IdentityRecords {
    const NAME: &'static str = "identity";
    type Bindings = IdentityBindings;
    type Members = (Mapping, Principal);
}
pub struct IdentityBindings;
impl BindingSet for IdentityBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for IdentityBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for IdentityBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for IdentityBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
pub struct Mapping;
impl Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (MappingFacts,);
    type Relations = (Target,);
}
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (Facts,);
    type Relations = ();
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
impl Record<MappingRow> for IdentityBindings {
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
pub struct Facts;
impl Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = PrincipalRow;
}
pub struct PrincipalRow {
    identity: u64,
}
impl Record<PrincipalRow> for IdentityBindings {
    fn record() -> impl Shape<PrincipalRow, Self> {
        field("Value", |row: &PrincipalRow| row.identity)
            .equality_queryable(true)
            .build(|identity, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow { identity })
            })
    }
}
pub type External =
    FieldSelection<IdentityRecords, Mapping, MappingFacts, WorthQueryExternalPrincipalIdentity>;
pub type Status =
    FieldSelection<IdentityRecords, Mapping, MappingFacts, WorthQueryPrincipalMappingStatus>;
pub type Value = FieldSelection<IdentityRecords, Principal, Facts, u64>;
pub struct Target;
impl Relation for Target {
    const NAME: &'static str = "Target";
    type To = Principal;
}
impl Target {
    pub fn reference() -> ApplicationRelationRef<Catalog, Self, Mapping, Principal> {
        fragment::relation(IdentityRecords, Mapping, Self)
    }
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<Catalog, Self> {
        fragment::entity(IdentityRecords, Self)
    }
}
