use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

fn declaration_for(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
) -> QuerySubscriptionDeclarationArtifact {
    declaration_for_basis(
        live_family,
        view_family,
        QuerySubscriptionBasisPosture::CurrentHead,
    )
}

fn declaration_for_basis(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    basis_posture: QuerySubscriptionBasisPosture,
) -> QuerySubscriptionDeclarationArtifact {
    let input = LiveQueryAdmissionArtifact::for_test_with_basis(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::FacadeLive,
        basis_posture,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    declare_query_subscription(selection, roomy_slice_budget()).unwrap()
}

mod basis;
mod denials;
mod family;
