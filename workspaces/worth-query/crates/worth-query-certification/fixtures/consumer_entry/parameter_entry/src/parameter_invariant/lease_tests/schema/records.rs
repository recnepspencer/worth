//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct LeaseRecords;
impl fragment::Fragment for LeaseRecords {
    const NAME: &'static str = "lease_records";
    type Bindings = LeaseRecordsBindings;
    type Members = (ExternalMapping, Principal);
}
pub struct LeaseRecordsBindings;
impl fragment::BindingSet for LeaseRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for LeaseRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<u64> for LeaseRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for LeaseRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
pub struct ExternalMapping;
impl fragment::Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<LeaseSchema, Self> {
        entity(LeaseRecords, Self)
    }
}
pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = (ParameterOwner,);
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<LeaseSchema, Self> {
        entity(LeaseRecords, Self)
    }
}
pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<LeaseRecords, ExternalMapping, Self> {
        aspect(LeaseRecords, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for LeaseRecordsBindings {
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
impl PrincipalIdentity {
    pub fn reference() -> AspectSelection<LeaseRecords, Principal, Self> {
        aspect(LeaseRecords, Principal, Self)
    }
}
pub struct PrincipalIdentityValue {
    pub principal_identity_field: u64,
}
impl fragment::Record<PrincipalIdentityValue> for LeaseRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalIdentityValue, Self> {
        fragment::field("PrincipalIdentityField", |row: &PrincipalIdentityValue| {
            row.principal_identity_field
        })
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
    LeaseRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type PrincipalIdentityField = FieldSelection<LeaseRecords, Principal, PrincipalIdentity, u64>;
pub type MappingStatusField = FieldSelection<
    LeaseRecords,
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
    pub fn reference() -> ApplicationRelationRef<LeaseSchema, Self, ExternalMapping, Principal> {
        relation(LeaseRecords, ExternalMapping, Self)
    }
}
pub struct ParameterOwner;
impl fragment::Relation for ParameterOwner {
    const NAME: &'static str = "ParameterOwner";
    type To = Parameter;
}
impl ParameterOwner {
    pub fn reference() -> ApplicationRelationRef<LeaseSchema, Self, Principal, Parameter> {
        relation(LeaseRecords, Principal, Self)
    }
}
