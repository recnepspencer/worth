use worth_query::facade::{domain, foundation, runtime};

use super::installed_operation_fixture::collection_impact::{
    impact_collection_invalidation_workspace, ImpactCollectionRead,
};
use super::installed_operation_fixture::configured_runtime;
use super::installed_operation_fixture::{
    consume_empty_invalidation_epoch as consume_empty_epoch, settle_native,
    InvalidationLease as Lease,
};
use super::installed_operation_fixture::{GeometryDomain, ReadFamily};

#[test]
fn two_affected_owners_and_four_leases_report_exact_k_plus_l_routing_work() {
    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Invalidation,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::DependencyImpact,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace("invalidation-indexed-k-plus-l")
        .unwrap();
    let (first_subject, first_candidate) = shared_pair(&mut workspace);
    let (second_subject, second_candidate) = shared_pair(&mut workspace);
    consume_empty_epoch(&mut workspace, &first_subject, &first_candidate);
    consume_empty_epoch(&mut workspace, &second_subject, &second_candidate);

    workspace
        .insert("Vertex", |mutation| {
            mutation.aspect("identity.id", "two-owner-update")
        })
        .unwrap();

    let first_subject_delivery = first_subject.drain(&mut workspace).unwrap();
    let first_candidate_delivery = first_candidate.drain(&mut workspace).unwrap();
    let second_subject_delivery = second_subject.drain(&mut workspace).unwrap();
    let second_candidate_delivery = second_candidate.drain(&mut workspace).unwrap();
    assert!(first_subject_delivery.shares_invalidation_epoch_with(&first_candidate_delivery));
    assert!(second_subject_delivery.shares_invalidation_epoch_with(&second_candidate_delivery));
    assert!(!first_subject_delivery.shares_invalidation_epoch_with(&second_subject_delivery));

    let deltas = [
        first_subject
            .consumer_invalidation_delta(first_subject_delivery)
            .unwrap(),
        first_candidate
            .consumer_invalidation_delta(first_candidate_delivery)
            .unwrap(),
        second_subject
            .consumer_invalidation_delta(second_subject_delivery)
            .unwrap(),
        second_candidate
            .consumer_invalidation_delta(second_candidate_delivery)
            .unwrap(),
    ];
    assert!(deltas[0].shares_epoch_with(&deltas[1]));
    assert!(deltas[2].shares_epoch_with(&deltas[3]));
    assert!(!deltas[0].shares_epoch_with(&deltas[2]));

    let exact_owner_work = deltas[0].epoch_counters().capability_index_lookups
        + deltas[2].epoch_counters().capability_index_lookups;
    let exact_fanout =
        deltas[0].epoch_counters().fanout_targets + deltas[2].epoch_counters().fanout_targets;
    let exact_lease_work = deltas
        .iter()
        .map(|delta| delta.counters().targeted_lease_deliveries)
        .sum::<usize>();
    assert_eq!(exact_owner_work, 2, "k affected projection owners");
    assert_eq!(exact_fanout, 4, "l exact lease targets");
    assert_eq!(exact_lease_work, 4, "one delivery per exact lease");
    assert!(deltas.iter().all(|delta| {
        delta.epoch_counters().live_target_candidates_visited == 1
            && delta.epoch_counters().installed_route_index_probes == 1
    }));
    let first = deltas[0].epoch_counters();
    let second = deltas[2].epoch_counters();
    assert_eq!(
        first.live_collection_index_probes + second.live_collection_index_probes,
        1
    );
    assert_eq!(
        first.installed_collection_index_probes + second.installed_collection_index_probes,
        1
    );
    assert_eq!(
        first.installed_target_candidates_selected + second.installed_target_candidates_selected,
        2
    );
    assert_eq!(
        first.installed_candidates_skipped + second.installed_candidates_skipped,
        0
    );
    assert_eq!(
        first.target_overlap_deduplications + second.target_overlap_deduplications,
        2
    );
}

#[test]
fn same_aspect_other_field_ordinary_consumers_do_not_enter_installed_owner_routing() {
    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Invalidation,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::DependencyImpact,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace("invalidation-same-aspect-field-routing")
        .unwrap();
    let inserted = workspace
        .insert("Vertex", |mutation| mutation.aspect("identity.id", "seed"))
        .unwrap();
    let entity = inserted.deltas()[0].entity_identity().clone();
    let (subject, candidate) = shared_pair(&mut workspace);
    consume_empty_epoch(&mut workspace, &subject, &candidate);
    let ordinary = (0..64)
        .map(|ordinal| ordinary_name_view(&mut workspace, ordinal))
        .collect::<Vec<_>>();

    workspace
        .update(entity, |mutation| {
            mutation.aspect("identity.id", "affected-installed-owner")
        })
        .unwrap();

    for view in ordinary {
        assert!(workspace.observe(&view).query_delivery_batches.is_empty());
    }
    let subject_delivery = subject.drain(&mut workspace).unwrap();
    let candidate_delivery = candidate.drain(&mut workspace).unwrap();
    let delta = subject
        .consumer_invalidation_delta(subject_delivery)
        .unwrap();
    assert!(candidate
        .consumer_invalidation_delta(candidate_delivery)
        .is_ok());
    assert_eq!(delta.epoch_counters().capability_index_lookups, 1);
    assert_eq!(delta.epoch_counters().live_target_candidates_visited, 1);
    assert_eq!(delta.epoch_counters().installed_route_index_probes, 1);
    assert_eq!(delta.epoch_counters().installed_candidates_skipped, 0);
}

#[test]
fn installed_ordering_dependency_routes_end_to_end_when_the_live_read_omits_it() {
    let mut workspace =
        impact_collection_invalidation_workspace("invalidation-installed-only-ordering-routing")
            .unwrap();
    let inserted = workspace
        .insert("Vertex", |mutation| {
            mutation
                .aspect("identity.id", "installed-ordering-row")
                .aspect("ordering.position", "first")
        })
        .unwrap();
    let entity = inserted.deltas()[0].entity_identity().clone();
    let live = match settle_installed_ordering(&mut workspace)
        .into_lifecycle()
        .promote(&mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("installed ordering subject did not promote"),
    };
    let candidate = settle_installed_ordering(&mut workspace).into_lifecycle();
    let (subject, candidate) = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared.into_leases(),
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("installed ordering sharing stopped: {}", stop.detail())
        }
    };
    assert!(subject.drain(&mut workspace).unwrap().delivery().is_empty());
    assert!(candidate
        .drain(&mut workspace)
        .unwrap()
        .delivery()
        .is_empty());

    workspace
        .update(entity, |mutation| {
            mutation.aspect("ordering.position", "second")
        })
        .unwrap();

    let delivery = subject.drain(&mut workspace).unwrap();
    let peer = candidate.drain(&mut workspace).unwrap();
    let delta = subject.consumer_invalidation_delta(delivery).unwrap();
    let peer_delta = candidate.consumer_invalidation_delta(peer).unwrap();
    assert_eq!(
        delta.impact().class(),
        domain::WorthQueryImpactClass::WindowShift
    );
    assert_eq!(
        delta.impact().affected_roles(),
        [
            domain::WorthQuerySemanticDependencyRole::Ordering,
            domain::WorthQuerySemanticDependencyRole::WindowBoundary,
        ]
    );
    assert!(delta.shares_epoch_with(&peer_delta));
    assert_eq!(
        delta.epoch_counters().installed_target_candidates_selected,
        1
    );
    assert_eq!(delta.epoch_counters().live_target_candidates_visited, 1);
    assert_eq!(delta.epoch_counters().capability_index_lookups, 1);
}

fn shared_pair(workspace: &mut runtime::WorthQueryWorkspace) -> (Lease, Lease) {
    let live = match settle_native(workspace).into_lifecycle().promote(workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("indexed routing subject did not promote"),
    };
    let candidate = settle_native(workspace).into_lifecycle();
    match live.share_with(candidate, workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared.into_leases(),
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("indexed routing sharing stopped: {}", stop.detail())
        }
    }
}

fn settle_installed_ordering(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ImpactCollectionRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ImpactCollectionRead)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
        .admit_execution_resources(
            (),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(
            consumer,
            worth_query::facade::read::project_facts().entity_identities(),
        )
        .unwrap()
        .settle()
        .unwrap()
}

fn ordinary_name_view(
    workspace: &mut runtime::WorthQueryWorkspace,
    ordinal: usize,
) -> runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape> {
    let request = foundation::DeclarativeLiveQueryRequest::new(
        "Vertex",
        foundation::DeclarativeLiveViewShape::table(),
    )
    .project(
        foundation::DeclarativeProjectionField::new(
            foundation::AspectFieldKey::from_authoring_parts("identity", "name").unwrap(),
        )
        .delivered_as("identity.name"),
    );
    let schema = runtime::QuerySchemaView::new(
        format!("same-aspect-name-{ordinal}"),
        [
            runtime::SchemaFieldView::new(
                foundation::AspectName::new("identity").unwrap(),
                foundation::FieldName::new("id").unwrap(),
                runtime::ScalarAspectType::String,
            ),
            runtime::SchemaFieldView::new(
                foundation::AspectName::new("identity").unwrap(),
                foundation::FieldName::new("name").unwrap(),
                runtime::ScalarAspectType::String,
            ),
        ],
        [],
    );
    workspace
        .live_view_request(format!("ordinary-name-{ordinal}"), request, schema)
        .unwrap()
}
