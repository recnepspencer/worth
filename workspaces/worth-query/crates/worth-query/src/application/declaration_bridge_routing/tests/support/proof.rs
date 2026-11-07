use crate::application::{
    worth_query_checked_declaration_bridge_routing_on_handle, WorthQueryDeclarationBridgeRouting,
    WorthQueryDeclarationBridgeRoutingChecked, WorthQueryDeclarationBridgeRoutingInput,
    WorthQueryDeclarationBridgeRoutingSupportStatus, WorthQueryDeclarationEnvelopeInput,
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationInput,
    WorthQueryDeclarationLegalityChecked, WorthQueryDeclarationLegalityInput,
    WorthQueryDeclarationReceiptInput, WorthQueryInstalledDomainDeclarationContext,
};
use crate::runtime::WorthQueryRuntimeFamilySupportStatus;

use super::domain::{GeometryDomain, GeometryWorld, RoutingInput};

pub(crate) fn checked_from_progressed<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: RoutingInput<F>,
) -> WorthQueryDeclarationBridgeRoutingChecked<GeometryDomain, RoutingInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    RoutingInput<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    let progressed = handle
        .declare_review_and_progress(declaration)
        .unwrap_or_else(|_| panic!("progression should admit"));
    let evidence = handle
        .describe_foundational(
            crate::application::WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                progressed.clone(),
            ),
        )
        .unwrap_or_else(|_| panic!("foundational evidence should materialize"));
    let route_checked = handle.plan_routes_checked(
        crate::application::WorthQueryDeclarationRoutePlanInput::admitted(progressed, evidence),
    );
    let receipt_checked = handle.receipt_routes_checked(
        WorthQueryDeclarationReceiptInput::route_checked(route_checked),
    );
    let envelope_checked = handle.envelope_routes_checked(
        WorthQueryDeclarationEnvelopeInput::receipt_checked(receipt_checked),
    );
    handle
        .route_bridge_continuation_checked(
            WorthQueryDeclarationBridgeRoutingInput::envelope_checked(envelope_checked),
        )
        .expect("fixture digest must fit")
}

pub(crate) fn routed_from_progressed<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: RoutingInput<F>,
) -> WorthQueryDeclarationBridgeRouting<GeometryDomain, RoutingInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    RoutingInput<F>: WorthQueryDeclarationInput<GeometryDomain>,
{
    let progressed = handle
        .declare_review_and_progress(declaration)
        .unwrap_or_else(|_| panic!("progression should admit"));
    let envelope = handle
        .envelope_routes_from_progressed(progressed)
        .unwrap_or_else(|_| panic!("envelope should admit"));
    handle
        .route_bridge_continuation(WorthQueryDeclarationBridgeRoutingInput::enveloped(envelope))
        .unwrap_or_else(|_| panic!("bridge routing should admit"))
}

pub(crate) fn checked_from_future_supported_runtime_test_posture<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: RoutingInput<F>,
) -> WorthQueryDeclarationBridgeRoutingChecked<GeometryDomain, RoutingInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    RoutingInput<F>: WorthQueryDeclarationInput<GeometryDomain, Family = F>,
{
    let canonical = handle
        .declare(declaration.clone())
        .unwrap_or_else(|_| panic!("future declaration should canonicalize"));
    let support_report = handle.family_support::<F>();
    let legal = match crate::application::review_declaration_legality(
        handle.handle_identity_digest(),
        WorthQueryDeclarationLegalityInput::new(
            canonical,
            support_report,
            F::legality_contract(),
            handle.retained_world_basis(),
            Some(WorthQueryRuntimeFamilySupportStatus::Supported),
            Some(WorthQueryRuntimeFamilySupportStatus::Supported),
        ),
    ) {
        WorthQueryDeclarationLegalityChecked::Legal(legal) => legal,
        WorthQueryDeclarationLegalityChecked::Illegal(_) => {
            panic!("future declaration should become legal under supported runtime test posture")
        }
    };
    let progressed = handle
        .progress_declaration(legal)
        .unwrap_or_else(|_| panic!("future progression should admit"));
    let evidence = handle
        .describe_foundational(
            crate::application::WorthQueryDeclarationFoundationalEvidenceInput::admitted_progression(
                progressed.clone(),
            ),
        )
        .unwrap_or_else(|_| panic!("future foundational evidence should materialize"));
    let route_checked = handle.plan_routes_checked(
        crate::application::WorthQueryDeclarationRoutePlanInput::admitted(progressed, evidence),
    );
    let receipt_checked = handle.receipt_routes_checked(
        WorthQueryDeclarationReceiptInput::route_checked(route_checked),
    );
    let envelope_checked = handle.envelope_routes_checked(
        WorthQueryDeclarationEnvelopeInput::receipt_checked(receipt_checked),
    );
    worth_query_checked_declaration_bridge_routing_on_handle(
        handle.handle_identity_digest(),
        handle.operating_context_identity_digest(),
        WorthQueryDeclarationBridgeRoutingSupportStatus::Admitted,
        WorthQueryDeclarationBridgeRoutingInput::envelope_checked(envelope_checked),
    )
    .expect("fixture digest must fit")
}

pub(crate) fn routed_from_future_supported_runtime_test_posture<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: RoutingInput<F>,
) -> WorthQueryDeclarationBridgeRouting<GeometryDomain, RoutingInput<F>>
where
    F: WorthQueryDeclarationFamilyMarker<GeometryDomain>,
    RoutingInput<F>: WorthQueryDeclarationInput<GeometryDomain, Family = F>,
{
    match checked_from_future_supported_runtime_test_posture(handle, declaration) {
        WorthQueryDeclarationBridgeRoutingChecked::Routed(routing) => routing,
        _ => panic!("future bridge routing should admit under supported runtime test posture"),
    }
}
