use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn every_family_maps_to_typed_delivery_intent() {
    let cases = [
        (
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionDeliveryIntent::ExactDetailReplacement,
        ),
        (
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionDeliveryIntent::OrderedMembershipDelta,
        ),
        (
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::KanbanGrouped),
            QuerySubscriptionDeliveryIntent::GroupedMembershipDelta,
        ),
        (
            LiveQueryFamily::Detail,
            Some(LiveViewShapeFamily::InspectorDetailObserved),
            QuerySubscriptionDeliveryIntent::InspectorFocusedDetailReplacement,
        ),
        (
            LiveQueryFamily::BoundedMaterialization,
            None,
            QuerySubscriptionDeliveryIntent::BoundedMaterializationMembershipDelta,
        ),
    ];

    for (live_family, view_family, expected_delivery) in cases {
        let input = LiveQueryAdmissionArtifact::for_test(
            live_family,
            view_family,
            QuerySubscriptionConstructionSource::FacadeLive,
        );
        let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
        let declaration = declare_query_subscription(selection, roomy_slice_budget()).unwrap();

        assert_eq!(declaration.delivery_intent(), &expected_delivery);
    }
}

#[test]
fn unsupported_delivery_intent_denies_before_declaration_digest() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    );
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    let budget = roomy_slice_budget().without_delivery_intent_support();

    let error = declare_query_subscription(selection, budget).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionDeclarationDenialKind::DeliveryIntentUnsupported
    );
    assert_eq!(error.counters().declaration_count(), 0);
    assert_eq!(error.counters().declaration_denial_count(), 1);
    assert_eq!(error.counters().delivery_intent_denial_count(), 1);
    assert_eq!(error.counters().declaration_digest_part_count(), 0);
}
