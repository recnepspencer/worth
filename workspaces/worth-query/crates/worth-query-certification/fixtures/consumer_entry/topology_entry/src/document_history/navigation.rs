//! Snapshot navigation admits its own typed action on the actual model grant.
use super::super::*;
use super::permission::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_capability::*, application_history::*, application_schema::*,
    portable_identity::WorthQueryPortableType, worth_query_operation,
};

pub struct NavigatePlanar<S>(PhantomData<fn() -> S>);
impl<S> WorthQueryPortableType for NavigatePlanar<S> {
    const PORTABLE_TYPE_NAME: &'static str = "NavigatePlanar";
}
impl<S: TopologySchemaBinding> ApplicationCapabilityMarkerIdentity for NavigatePlanar<S> {
    type Schema = S;
    const IDENTIFIER: &'static str = "NavigatePlanar";
}
impl<S: TopologySchemaBinding> HistoryPermission<S> for NavigatePlanar<S> {
    const ACTION: &'static str = "navigate";
    const GRANT_KIND: &'static str = "navigation";
}
worth_query_operation!(pub NavigatePlanarOperation for Schema: TopologySchemaBinding, input PlanarOutputReadInputBinding);

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
pub(super) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
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
            BodyKey::at("BodyKey"),
        )
        .capability(contract::<S, NavigatePlanar<S>, NavigatePlanarOperation>())
}
