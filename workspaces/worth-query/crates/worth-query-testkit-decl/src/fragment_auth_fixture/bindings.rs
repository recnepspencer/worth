//! Authentication values use the same binding set as the stored fragment rows.
use super::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrincipalData {
    pub id: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappingData {
    pub external: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for Bindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Record<PrincipalData> for Bindings {
    fn record() -> impl Shape<PrincipalData, Self> {
        field("id", |row: &PrincipalData| row.id)
            .equality_queryable(true)
            .read_only()
            .build(|id, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalData { id })
            })
    }
}
impl Record<MappingData> for Bindings {
    fn record() -> impl Shape<MappingData, Self> {
        field("external", |row: &MappingData| row.external.clone())
            .equality_queryable(true)
            .unique()
            .and("status", |row: &MappingData| row.status)
            .build(|external, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingData { external, status })
            })
    }
}
