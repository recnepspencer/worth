use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

use super::*;

pub(super) fn table_declaration() -> QuerySubscriptionDeclarationArtifact {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::Direct,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(live, roomy_budget()).unwrap();
    declare_query_subscription(selection, roomy_slice_budget()).unwrap()
}

pub(super) fn table_lowering() -> BridgeSubscriptionLoweringPlan {
    lower_query_subscription_to_bridge(table_declaration(), roomy_lowering_budget()).unwrap()
}

#[derive(Debug)]
pub(super) struct CertifiedSubscriptionIdentity {
    pub(super) declaration_digest: String,
    pub(super) basis_request_digest: String,
}

pub(super) fn certified_subscription_identity(
    policy: &str,
    tenant: &str,
    proof: &str,
) -> CertifiedSubscriptionIdentity {
    let live = LiveQueryAdmissionArtifact::for_test_with_context(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::SavedExactReuse,
        QuerySubscriptionBasisPosture::CurrentHead,
        QuerySubscriptionFutureSelection::ordinary().expect("fixture digest must fit"),
        Some(policy.to_string()),
        Some(tenant.to_string()),
        Some(proof.to_string()),
        QuerySubscriptionRelationshipProofPosture::Admitted,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(live, roomy_budget()).unwrap();
    let declaration = declare_query_subscription(selection, roomy_slice_budget()).unwrap();
    let declaration_digest = declaration.declaration_projection().label().to_string();
    let lowering =
        lower_query_subscription_to_bridge(declaration, roomy_lowering_budget()).unwrap();
    CertifiedSubscriptionIdentity {
        declaration_digest,
        basis_request_digest: lowering
            .basis_request()
            .basis_binding_projection()
            .label()
            .to_string(),
    }
}
