//! Snapshot navigation has its own current capability; READ remains separate.
use super::super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_capability::*, application_history::*, application_schema::*,
    portable_identity::WorthQueryPortableType, worth_query_operation, worth_query_operation_reads,
};
mod contract;
mod grant_status;
mod grants;
pub use contract::bind_navigation_grant;
pub use grant_status::*;
pub use grants::*;

macro_rules! marker {
    ($name:ident, $identity:ident, $reference:ident) => {
        pub struct $name<S>(PhantomData<fn() -> S>);
        impl<S> WorthQueryPortableType for $name<S> {
            const PORTABLE_TYPE_NAME: &'static str = stringify!($name);
        }
        impl<S: TopologySchemaBinding> $identity for $name<S> {
            type Schema = S;
            const IDENTIFIER: &'static str = stringify!($name);
        }
        impl<S: TopologySchemaBinding> $name<S> {
            pub fn reference() -> $reference<S, Self> {
                $reference::from_declaration()
            }
        }
    };
}
marker!(
    NavigatePlanar,
    ApplicationCapabilityMarkerIdentity,
    ApplicationCapabilityRef
);
marker!(
    PlanarNavigationContext,
    ApplicationCapabilityContextMarkerIdentity,
    ApplicationCapabilityContextRef
);
marker!(
    PlanarNavigationProvenance,
    ApplicationCapabilityProvenanceMarkerIdentity,
    ApplicationCapabilityProvenanceRef
);
worth_query_operation!(pub NavigatePlanarOperation for Schema: TopologySchemaBinding, input PlanarOutputReadInputBinding);
worth_query_operation_reads!(NavigatePlanarOperation => [BodyKey]);

pub struct PlanarHistoryNavigation<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> ApplicationHistoryNavigationBinding<S, PlanarDocumentHistory<S>>
    for PlanarHistoryNavigation<S>
{
    const IDENTITY: &'static str = "worth.query.certification.planar-history-navigation.v1";
    type Capability = NavigatePlanar<S>;
    type Input = PlanarOutputRead;
    type InputBinding = PlanarOutputReadInputBinding;
    type Operation = NavigatePlanarOperation;
    fn input(access: &PlanarOutputRead) -> PlanarOutputRead {
        access.clone()
    }
}
impl<S: TopologySchemaBinding> ApplicationCapabilityRequest<S, NavigatePlanar<S>>
    for PlanarOutputRead
{
    type Scope = Body;
    type Context = PlanarNavigationContext<S>;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<S, Body, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                BodyKey::reference(),
                encoded(self.body_key.clone()),
            ),
            encoded("navigate".to_owned()),
            encoded("model-history".to_owned()),
            ApplicationCapabilityRequestContext::new(PlanarNavigationContext::<S>::reference()),
        ))
    }
}
fn encoded(value: String) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("navigation values are strings")
}
pub(super) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    grant_status::declare(grants::declare(schema))
        .capability_context(PlanarNavigationContext::<S>::reference())
        .capability_provenance(PlanarNavigationProvenance::<S>::reference())
        .operation(
            NavigatePlanarOperation::reference::<S>()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(NavigatePlanarOperation::reference::<S>(), 512)
        .operation_read_field(
            NavigatePlanarOperation::reference::<S>(),
            BodyKey::reference(),
        )
        .capability(contract::contract::<S>())
}
