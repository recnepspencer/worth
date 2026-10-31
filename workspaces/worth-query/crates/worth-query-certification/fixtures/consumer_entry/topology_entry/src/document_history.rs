//! History is authored beside the real READ and ordinary initial adjustment.
use super::*;
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_history::{ApplicationHistoryBinding, ApplicationHistoryDefinition},
    application_query::ApplicationQueryDisclosureContract,
    application_schema::ApplicationSchemaDeclarationBuilder,
    portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity},
};

pub struct PlanarDocumentHistory<S>(PhantomData<fn() -> S>);
pub struct PlanarSourceDocumentHistory<S>(PhantomData<fn() -> S>);
impl<S: TopologySchemaBinding> WorthQueryPortableType for PlanarSourceDocumentHistory<S> {
    const PORTABLE_TYPE_NAME: &'static str = "PlanarSourceDocumentHistory";
    const PORTABLE_TYPE_IDENTITY: WorthQueryPortableTypeIdentity =
        WorthQueryPortableTypeIdentity::declared(
            "worth.query.certification.planar-source-document-history.v1",
        );
}
impl<S: TopologySchemaBinding> ApplicationHistoryBinding<S> for PlanarSourceDocumentHistory<S> {
    type Access = PlanarRead;
    const IDENTITY: &'static str = "PlanarSourceDocumentHistory";
}
impl<S: TopologySchemaBinding> WorthQueryPortableType for PlanarDocumentHistory<S> {
    const PORTABLE_TYPE_NAME: &'static str = "PlanarDocumentHistory";
    const PORTABLE_TYPE_IDENTITY: WorthQueryPortableTypeIdentity =
        WorthQueryPortableTypeIdentity::declared(
            "worth.query.certification.planar-document-history.v1",
        );
}
impl<S: TopologySchemaBinding> ApplicationHistoryBinding<S> for PlanarDocumentHistory<S> {
    type Access = PlanarOutputRead;
    const IDENTITY: &'static str = "PlanarDocumentHistory";
}
pub(super) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .application_history(
            ApplicationHistoryDefinition::<S, PlanarDocumentHistory<S>>::for_binding(
                ApplicationQueryDisclosureContract::public(),
            )
            .operation::<PlanarInitialAdjustmentBinding<S>>("Set model Y"),
        )
        .application_history(
            ApplicationHistoryDefinition::<S, PlanarSourceDocumentHistory<S>>::for_binding(
                ApplicationQueryDisclosureContract::public(),
            )
            .operation::<PlanarInitialAdjustmentBinding<S>>("Set model Y"),
        )
}
