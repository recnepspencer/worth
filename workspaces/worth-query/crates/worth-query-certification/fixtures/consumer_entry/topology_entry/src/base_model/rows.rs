//! Shared value bindings and checked rows contain no entity or relation membership.

use super::{PositiveLength, TopologyLengthBinding};
use worth_query_decl::facade::{
    application_schema::{fragment, *},
    authentication::*,
};
pub struct TopologyRecordsBindings;
impl fragment::BindingSet for TopologyRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for TopologyRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for TopologyRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for TopologyRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<String> for TopologyRecordsBindings {
    type Binding = worth_query_decl::facade::application_schema::StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<PositiveLength> for TopologyRecordsBindings {
    type Binding = TopologyLengthBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for TopologyRecordsBindings {
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
pub struct PrincipalFactsValue {
    pub principal_identity: u64,
}
impl fragment::Record<PrincipalFactsValue> for TopologyRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalFactsValue, Self> {
        fragment::field("PrincipalIdentity", |row: &PrincipalFactsValue| {
            row.principal_identity
        })
        .build(|principal_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalFactsValue { principal_identity })
        })
    }
}
pub struct PlanarPositionValue {
    pub body_key: String,
    pub position_x: PositiveLength,
    pub position_y: PositiveLength,
}
impl fragment::Record<PlanarPositionValue> for TopologyRecordsBindings {
    fn record() -> impl fragment::Shape<PlanarPositionValue, Self> {
        fragment::field("BodyKey", |row: &PlanarPositionValue| row.body_key.clone())
            .and("PositionX", |row: &PlanarPositionValue| row.position_x)
            .and("PositionY", |row: &PlanarPositionValue| row.position_y)
            .build(
                |body_key, position_x, position_y, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(PlanarPositionValue {
                        body_key,
                        position_x,
                        position_y,
                    })
                },
            )
    }
}
pub struct GeometryValue {
    pub length: PositiveLength,
}
impl fragment::Record<GeometryValue> for TopologyRecordsBindings {
    fn record() -> impl fragment::Shape<GeometryValue, Self> {
        fragment::field("Length", |row: &GeometryValue| row.length).build(
            |length, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(GeometryValue { length })
            },
        )
    }
}
