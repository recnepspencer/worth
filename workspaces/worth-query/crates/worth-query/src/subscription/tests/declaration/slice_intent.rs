use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

fn declared_kinds(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
) -> Vec<QuerySubscriptionSliceKind> {
    let input = LiveQueryAdmissionArtifact::for_test(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    let declaration = declare_query_subscription(selection, roomy_slice_budget()).unwrap();
    declaration
        .slice_intent()
        .parts()
        .iter()
        .map(|part| part.kind().clone())
        .collect()
}

#[test]
fn detail_slice_intent_is_projected_only() {
    let kinds = declared_kinds(LiveQueryFamily::Detail, None);

    assert_eq!(
        kinds,
        vec![
            QuerySubscriptionSliceKind::AuthorizedProjection,
            QuerySubscriptionSliceKind::AuthorizedProjection
        ]
    );
}

#[test]
fn collection_slice_intent_includes_membership_projection_and_ordering() {
    let kinds = declared_kinds(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );

    assert_eq!(
        kinds,
        vec![
            QuerySubscriptionSliceKind::AuthorizedProjection,
            QuerySubscriptionSliceKind::AuthorizedProjection,
            QuerySubscriptionSliceKind::Membership,
            QuerySubscriptionSliceKind::Ordering
        ]
    );
}

#[test]
fn grouped_slice_intent_includes_grouping_and_metadata() {
    let kinds = declared_kinds(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
    );

    assert_eq!(
        kinds,
        vec![
            QuerySubscriptionSliceKind::AuthorizedProjection,
            QuerySubscriptionSliceKind::AuthorizedProjection,
            QuerySubscriptionSliceKind::Membership,
            QuerySubscriptionSliceKind::Ordering,
            QuerySubscriptionSliceKind::Grouping,
            QuerySubscriptionSliceKind::ViewShapeMetadata
        ]
    );
}

#[test]
fn inspector_slice_intent_includes_view_shape_metadata() {
    let kinds = declared_kinds(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::InspectorDetailFocused),
    );

    assert_eq!(
        kinds,
        vec![
            QuerySubscriptionSliceKind::AuthorizedProjection,
            QuerySubscriptionSliceKind::AuthorizedProjection,
            QuerySubscriptionSliceKind::ViewShapeMetadata
        ]
    );
}

#[test]
fn bounded_materialization_slice_intent_includes_relation_scope() {
    let kinds = declared_kinds(LiveQueryFamily::BoundedMaterialization, None);

    assert_eq!(
        kinds,
        vec![
            QuerySubscriptionSliceKind::AuthorizedProjection,
            QuerySubscriptionSliceKind::AuthorizedProjection,
            QuerySubscriptionSliceKind::Membership,
            QuerySubscriptionSliceKind::Ordering,
            QuerySubscriptionSliceKind::RelationScope
        ]
    );
}
