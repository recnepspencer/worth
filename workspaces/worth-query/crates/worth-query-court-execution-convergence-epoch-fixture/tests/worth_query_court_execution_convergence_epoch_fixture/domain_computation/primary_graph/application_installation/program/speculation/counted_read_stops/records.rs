//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct PreviewRecords;
impl fragment::Fragment for PreviewRecords {
    const NAME: &'static str = "preview_records";
    type Bindings = PreviewRecordsBindings;
    type Members = (PreviewEntity, PreviewMapping);
}
pub struct PreviewRecordsBindings;
impl fragment::BindingSet for PreviewRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for PreviewRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for PreviewRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for PreviewRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
pub struct PreviewEntity;
impl fragment::Entity for PreviewEntity {
    const NAME: &'static str = "PreviewEntity";
    type Aspects = (PrincipalAspect,);
    type Relations = ();
}

pub struct PreviewMapping;
impl fragment::Entity for PreviewMapping {
    const NAME: &'static str = "PreviewMapping";
    type Aspects = (MappingAspect,);
    type Relations = (MappingTarget,);
}

pub struct MappingAspect;
impl fragment::Aspect for MappingAspect {
    const NAME: &'static str = "MappingAspect";
    type Row = MappingAspectValue;
}

pub struct MappingAspectValue {
    pub mapping_identity: WorthQueryExternalPrincipalIdentity,
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<MappingAspectValue> for PreviewRecordsBindings {
    fn record() -> impl fragment::Shape<MappingAspectValue, Self> {
        fragment::field("MappingIdentity", |row: &MappingAspectValue| {
            row.mapping_identity.clone()
        })
        .and("MappingStatus", |row: &MappingAspectValue| {
            row.mapping_status
        })
        .build(
            |mapping_identity, mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(MappingAspectValue {
                    mapping_identity,
                    mapping_status,
                })
            },
        )
    }
}
pub struct PrincipalAspect;
impl fragment::Aspect for PrincipalAspect {
    const NAME: &'static str = "PrincipalAspect";
    type Row = PrincipalAspectValue;
}

pub struct PrincipalAspectValue {
    pub principal_identity: u64,
}
impl fragment::Record<PrincipalAspectValue> for PreviewRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalAspectValue, Self> {
        fragment::field("PrincipalIdentity", |row: &PrincipalAspectValue| {
            row.principal_identity
        })
        .build(|principal_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalAspectValue { principal_identity })
        })
    }
}
pub type MappingIdentity = FieldSelection<
    PreviewRecords,
    PreviewMapping,
    MappingAspect,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatus =
    FieldSelection<PreviewRecords, PreviewMapping, MappingAspect, WorthQueryPrincipalMappingStatus>;
pub type PrincipalIdentity = FieldSelection<PreviewRecords, PreviewEntity, PrincipalAspect, u64>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = PreviewEntity;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<PreviewSchema, Self, PreviewMapping, PreviewEntity>
    {
        relation(PreviewRecords, PreviewMapping, Self)
    }
}
