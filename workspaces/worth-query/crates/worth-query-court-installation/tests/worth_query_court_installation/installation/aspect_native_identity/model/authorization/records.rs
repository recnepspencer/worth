//! Native membership for principal-bound authorization budget courts.
use super::*;
use crate::installation::aspect_native_identity::model::records::{Records, Set};
use fragment::{Aspect, Entity, Fragment, Relation};
pub struct Authorization;
impl Fragment for Authorization {
    const NAME: &'static str = "authorization";
    type Bindings = Set;
    type Members = (Records<2, 1, false>, Mapping, Principal);
}
pub struct Mapping;
pub struct Principal;
pub struct External;
pub struct PrincipalFacts;
pub struct MappingTarget;
impl Entity for Mapping {
    const NAME: &'static str = "Mapping";
    type Aspects = (External,);
    type Relations = (MappingTarget,);
}
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Relation for MappingTarget {
    const NAME: &'static str = "PrincipalTarget";
    type To = Principal;
}
impl Aspect for External {
    const NAME: &'static str = "External";
    type Row = MappingRow;
}
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalRow;
}
pub struct MappingRow {
    external: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
}
pub struct PrincipalRow(u64);
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for Set {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for Set {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl fragment::Record<MappingRow> for Set {
    fn record() -> impl fragment::Shape<MappingRow, Self> {
        fragment::field("ExternalId", |r: &MappingRow| r.external.clone())
            .equality_queryable(true)
            .read_only()
            .and("Status", |r: &MappingRow| r.status)
            .build(|external, status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingRow { external, status })
            })
    }
}
impl fragment::Record<PrincipalRow> for Set {
    fn record() -> impl fragment::Shape<PrincipalRow, Self> {
        fragment::field("Identity", |r: &PrincipalRow| r.0)
            .equality_queryable(true)
            .read_only()
            .build(|value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalRow(value))
            })
    }
}
