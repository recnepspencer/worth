use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn every_query_family_lowers_to_explicit_bridge_family_and_slices() {
    let cases = [
        (
            LiveQueryFamily::Detail,
            None,
            BridgeSubscriptionDeclarationFamilyKind::DetailExact,
            vec![BridgeSubscriptionSliceKind::ProjectedField; 2],
        ),
        (
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            BridgeSubscriptionDeclarationFamilyKind::CollectionMembership,
            vec![
                BridgeSubscriptionSliceKind::ProjectedField,
                BridgeSubscriptionSliceKind::ProjectedField,
                BridgeSubscriptionSliceKind::Membership,
                BridgeSubscriptionSliceKind::Ordering,
            ],
        ),
        (
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::KanbanGrouped),
            BridgeSubscriptionDeclarationFamilyKind::CollectionMembership,
            vec![
                BridgeSubscriptionSliceKind::ProjectedField,
                BridgeSubscriptionSliceKind::ProjectedField,
                BridgeSubscriptionSliceKind::Membership,
                BridgeSubscriptionSliceKind::Ordering,
                BridgeSubscriptionSliceKind::Grouping,
                BridgeSubscriptionSliceKind::ViewMetadata,
            ],
        ),
        (
            LiveQueryFamily::Detail,
            Some(LiveViewShapeFamily::InspectorDetailFocused),
            BridgeSubscriptionDeclarationFamilyKind::DetailExact,
            vec![
                BridgeSubscriptionSliceKind::ProjectedField,
                BridgeSubscriptionSliceKind::ProjectedField,
                BridgeSubscriptionSliceKind::ViewMetadata,
            ],
        ),
        (
            LiveQueryFamily::BoundedMaterialization,
            None,
            BridgeSubscriptionDeclarationFamilyKind::CollectionMembership,
            vec![
                BridgeSubscriptionSliceKind::ProjectedField,
                BridgeSubscriptionSliceKind::ProjectedField,
                BridgeSubscriptionSliceKind::Membership,
                BridgeSubscriptionSliceKind::Ordering,
                BridgeSubscriptionSliceKind::RelationScope,
            ],
        ),
    ];

    for (live_family, view_family, expected_family, expected_slices) in cases {
        let declaration = declaration_for(live_family, view_family);
        let plan =
            lower_query_subscription_to_bridge(declaration, roomy_lowering_budget()).unwrap();

        assert_eq!(plan.bridge_family(), &expected_family);
        assert_eq!(plan.bridge_slices(), expected_slices.as_slice());
        assert_eq!(plan.counters().bridge_lowering_count(), 1);
        assert_eq!(plan.counters().bridge_family_registry_lookup_count(), 1);
        assert_eq!(
            plan.counters().bridge_slice_count(),
            expected_slices.len() as u64
        );
        assert_eq!(
            plan.counters().bridge_slice_registry_lookup_count(),
            expected_slices.len() as u64
        );
        assert_eq!(plan.counters().basis_binding_request_count(), 1);
        assert_eq!(plan.counters().signal_strategy_request_count(), 1);
    }
}

#[test]
fn equivalent_declarations_lower_to_identical_bridge_digest() {
    let declaration = |source| {
        let input = LiveQueryAdmissionArtifact::for_test(LiveQueryFamily::Detail, None, source)
            .expect("fixture digest must fit");
        let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
        declare_query_subscription(selection, roomy_slice_budget()).unwrap()
    };

    let direct = lower_query_subscription_to_bridge(
        declaration(QuerySubscriptionConstructionSource::Direct),
        roomy_lowering_budget(),
    )
    .unwrap();
    let saved = lower_query_subscription_to_bridge(
        declaration(QuerySubscriptionConstructionSource::SavedExactReuse),
        roomy_lowering_budget(),
    )
    .unwrap();

    assert_eq!(
        direct.bridge_declaration_projection().label(),
        saved.bridge_declaration_projection().label()
    );
}
