use crate::application::{
    WorthQueryDeclarationEnvelopeInput, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationInput, WorthQueryDeclarationReceiptInput,
    WorthQueryDeclarationRelationalRoutingChecked, WorthQueryDeclarationRelationalRoutingInput,
    WorthQueryInstalledDomainDeclarationContext,
};

use super::domain::{GeometryDomain, GeometryWorld, RoutingInput};

pub(crate) fn checked_from_progressed<F>(
    handle: &WorthQueryInstalledDomainDeclarationContext<GeometryDomain, GeometryWorld>,
    declaration: RoutingInput<F>,
) -> WorthQueryDeclarationRelationalRoutingChecked<GeometryDomain, RoutingInput<F>>
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
        .route_relational_truth_checked(
            WorthQueryDeclarationRelationalRoutingInput::envelope_checked(envelope_checked),
        )
        .expect("fixture digest must fit")
}
