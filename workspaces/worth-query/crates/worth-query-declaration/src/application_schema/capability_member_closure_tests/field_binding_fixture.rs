use super::{Facts, Grant, Resource, ResourceFacts};
use crate::application_capability::ApplicationCapabilityFieldBinding;
use crate::application_schema::{fragment, U64ApplicationValueBinding, ValueDecodeAdmission};
pub(super) struct CapabilityModel;
impl fragment::Fragment for CapabilityModel {
    const NAME: &'static str = "CapabilityModel";
    type Bindings = CapabilityBindings;
    type Members = (Grant, Resource);
}
pub(super) struct CapabilityBindings;
impl fragment::BindingSet for CapabilityBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for CapabilityBindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl fragment::Entity for Grant {
    const NAME: &'static str = "Grant";
    type Aspects = (Facts,);
    type Relations = ();
}
impl fragment::Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = GrantRow;
}
pub(super) struct GrantRow(u64, u64, u64, u64, u64, u64, u64, u64, u64, u64, u64);
impl fragment::Record<GrantRow> for CapabilityBindings {
    fn record() -> impl fragment::Shape<GrantRow, Self> {
        fragment::field("Action", |row: &GrantRow| row.0)
            .equality_queryable(true)
            .and("Amount", |row: &GrantRow| row.1)
            .equality_queryable(true)
            .and("Field", |row: &GrantRow| row.2)
            .equality_queryable(true)
            .and("OtherAction", |row: &GrantRow| row.3)
            .equality_queryable(true)
            .and("Purpose", |row: &GrantRow| row.4)
            .equality_queryable(true)
            .and("Status", |row: &GrantRow| row.5)
            .equality_queryable(true)
            .and("DelegationLimit", |row: &GrantRow| row.6)
            .equality_queryable(true)
            .and("Identity", |row: &GrantRow| row.7)
            .equality_queryable(true)
            .and("ValidFrom", |row: &GrantRow| row.8)
            .equality_queryable(true)
            .and("ValidThrough", |row: &GrantRow| row.9)
            .equality_queryable(true)
            .and("Workflow", |row: &GrantRow| row.10)
            .equality_queryable(true)
            .build(
                |v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(GrantRow(
                        v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10,
                    ))
                },
            )
    }
}
impl fragment::Entity for Resource {
    const NAME: &'static str = "Resource";
    type Aspects = (ResourceFacts,);
    type Relations = ();
}
impl fragment::Aspect for ResourceFacts {
    const NAME: &'static str = "ResourceFacts";
    type Row = ResourceRow;
}
pub(super) struct ResourceRow(u64);
impl fragment::Record<ResourceRow> for CapabilityBindings {
    fn record() -> impl fragment::Shape<ResourceRow, Self> {
        fragment::field("ResourceWorkflow", |row: &ResourceRow| row.0)
            .equality_queryable(true)
            .build(|v0, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ResourceRow(v0))
            })
    }
}
pub(super) fn field(
    name: &'static str,
) -> fragment::FieldSelection<CapabilityModel, Grant, Facts, u64> {
    fragment::FieldSelection::at(name)
}
pub(super) fn binding(name: &'static str) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(field(name)).unwrap()
}
pub(super) fn resource_binding(name: &'static str) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(fragment::FieldSelection::<
        CapabilityModel,
        Resource,
        ResourceFacts,
        u64,
    >::at(name))
    .unwrap()
}
pub(super) fn encoded(
    value: u64,
) -> crate::application_schema::ApplicationEncodedScalarValue<U64ApplicationValueBinding> {
    crate::application_schema::ApplicationEncodedScalarValue::try_new(value).unwrap()
}
