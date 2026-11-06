//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct PrincipalRecords;
impl fragment::Fragment for PrincipalRecords {
    const NAME: &'static str = "principal_records";
    type Bindings = PrincipalRecordsBindings;
    type Members = (ExternalMapping, Principal);
}
pub struct PrincipalRecordsBindings;
impl fragment::BindingSet for PrincipalRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for PrincipalRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<u64> for PrincipalRecordsBindings {
    type Binding = worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for PrincipalRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
#[derive(Clone, Copy, Debug)]
pub struct ExternalMapping;
impl fragment::Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}

#[derive(Clone, Copy, Debug)]
pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = ();
}

pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}

pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for PrincipalRecordsBindings {
    fn record() -> impl fragment::Shape<ExternalIdentityValue, Self> {
        fragment::field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
        .build(
            |external_identity_field, mapping_status_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status_field,
                })
            },
        )
    }
}
pub struct PrincipalIdentity;
impl fragment::Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = PrincipalIdentityValue;
}

pub struct PrincipalIdentityValue {
    pub principal_identity_field: u64,
}
impl fragment::Record<PrincipalIdentityValue> for PrincipalRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalIdentityValue, Self> {
        fragment::field("PrincipalIdentityField", |row: &PrincipalIdentityValue| {
            row.principal_identity_field
        })
        .unique()
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalIdentityValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub type ExternalIdentityField = FieldSelection<
    PrincipalRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type PrincipalIdentityField =
    FieldSelection<PrincipalRecords, Principal, PrincipalIdentity, u64>;
pub type MappingStatusField = FieldSelection<
    PrincipalRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<PrimaryGraphCompositionSchema, Self, ExternalMapping, Principal>
    {
        relation(PrincipalRecords, ExternalMapping, Self)
    }
}
