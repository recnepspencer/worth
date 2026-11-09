//! One reusable installation fixture fragment, independent of its installation root.
use super::TestOperation;
use worth_query_declaration::facade::application_schema::{fragment::*, *};
use worth_query_declaration::facade::authentication::*;

pub(super) struct InstallationFields;
impl Fragment for InstallationFields {
    const NAME: &'static str = "installation_fields";
    type Bindings = InstallationBindings;
    type Members = (FixtureEntity,);
}
pub(super) struct InstallationBindings;
impl BindingSet for InstallationBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl Scalar<u64> for InstallationBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for InstallationBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for InstallationBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
pub(super) struct FixtureEntity;
impl Entity for FixtureEntity {
    const NAME: &'static str = "TestEntity";
    type Aspects = (FixtureIdentityAspect,);
    type Relations = (super::MappingTarget,);
}
pub(super) struct FixtureIdentityAspect;
impl Aspect for FixtureIdentityAspect {
    const NAME: &'static str = "IdentityAspect";
    type Row = InstallationIdentity;
}
pub(super) struct InstallationIdentity {
    external: WorthQueryExternalPrincipalIdentity,
    status: WorthQueryPrincipalMappingStatus,
    principal: u64,
}
impl Record<InstallationIdentity> for InstallationBindings {
    fn record() -> impl Shape<InstallationIdentity, Self> {
        field("ExternalIdentityField", |r: &InstallationIdentity| {
            r.external.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |r: &InstallationIdentity| r.status)
        .and("PrincipalIdentityField", |r: &InstallationIdentity| {
            r.principal
        })
        .equality_queryable(true)
        .build(
            |external, status, principal, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(InstallationIdentity {
                    external,
                    status,
                    principal,
                })
            },
        )
    }
}
pub(super) type FixtureMappingStatusField = FieldSelection<
    InstallationFields,
    FixtureEntity,
    FixtureIdentityAspect,
    WorthQueryPrincipalMappingStatus,
>;
pub(super) type FixturePrincipalIdentityField =
    FieldSelection<InstallationFields, FixtureEntity, FixtureIdentityAspect, u64>;
pub(super) type TestEntity = FixtureEntity;
impl<Schema> OperationCreates<TestOperation<Schema>> for FixtureEntity {}
impl Relation for super::MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = FixtureEntity;
}
