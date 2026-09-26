use worth_foundational::facade::{AspectValue, InternedString};

use crate::facade::config::CascadeDeletePolicy;
use crate::facade::history::BranchId;
use crate::facade::runtime::RelationalRuntimeApi;
use crate::identity::data::KindId;
use crate::tests::support::{
    aspect_field_locator, aspect_key, create_entity, create_relation, delete_relation_on_branch,
    field_key, runtime_with_test_schema, test_schema_registry, AspectSchemaFixture,
};
use crate::transactions::data::RecordRef;

use super::{
    RelationalAuthorizationEffectTarget, RelationalAuthorizationObservationCounters,
    RelationalAuthorizationObservationDenial, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPathPlan, RelationalAuthorizationPlanDenial,
    RelationalAuthorizationPredicate, RelationalAuthorizationTraversal,
    RelationalAuthorizationTraversalDirection,
};

mod anchored_paths;
mod durable_dependencies;
mod exact_adjacency;
mod freshness;
mod strictly_greater;
mod witnesses;

const ENTITY_KIND: KindId = KindId(1);
const RELATION_KIND: KindId = KindId(2);

#[test]
fn actual_snapshot_observation_mints_exact_neutral_evidence() {
    let fixture = authorization_fixture();
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let plan = allow_plan(
        snapshot,
        fixture.principal,
        fixture.scope,
        [RelationalAuthorizationEffectTarget::record(
            RecordRef::Entity(fixture.scope),
        )],
    );
    let evidence = fixture
        .runtime
        .observe_authorization(plan)
        .expect("installed path should be observable");

    assert_ne!(evidence.observation_identity().bytes(), &[0; 32]);
    assert_eq!(evidence.principal(), fixture.principal);
    assert_eq!(evidence.scope(), fixture.scope);
    assert!(evidence.paths()[0].matched());
    let witness = evidence.paths()[0]
        .witness()
        .expect("a matched path retains one complete causal witness");
    assert_eq!(witness.entity_at(0), Some(fixture.principal));
    assert_eq!(witness.entity_at(2), Some(fixture.scope));
    assert!(evidence.paths()[0].exhaustive());
    assert_eq!(evidence.paths()[0].adjacency_lists().len(), 2);
    assert_eq!(
        evidence.counters(),
        RelationalAuthorizationObservationCounters {
            paths_evaluated: 1,
            adjacency_lists_read: 2,
            adjacency_edges_inspected: 2,
            relation_records_inspected: 2,
            entity_records_inspected: 5,
            predicate_fields_inspected: 1,
            relation_join_index_lookups: 0,
            relation_join_candidates_inspected: 0,
            maximum_frontier_width: 1,
            reconstructive_graph_scans: 0,
            reconstructive_relation_records_scanned: 0,
        }
    );
}

#[test]
fn parallel_matching_paths_remain_neutral_graph_observations() {
    let fixture = authorization_fixture();
    create_relation(
        &fixture.runtime,
        fixture.principal,
        fixture.scope,
        "initiated-payment",
    );
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let plan = role_and_direct_path_plan(snapshot, fixture.principal, fixture.scope);

    let evidence = fixture
        .runtime
        .observe_authorization(plan)
        .expect("the snapshot can be observed");

    assert!(evidence.paths()[0].matched());
    assert!(evidence.paths()[1].matched());
    assert!(evidence.paths().iter().all(|path| path.exhaustive()));
}

#[test]
fn revocation_changes_only_new_snapshot_authority() {
    let fixture = authorization_fixture();
    let before = fixture.runtime.visibility_authority().snapshot();
    let before_evidence = fixture
        .runtime
        .observe_authorization(allow_plan(
            before.clone(),
            fixture.principal,
            fixture.scope,
            [],
        ))
        .expect("pre-revocation observation");
    let revocation = delete_relation_on_branch(
        &fixture.runtime,
        fixture.role_scope_relation,
        BranchId("main".to_string()),
    );

    let after_evidence = fixture
        .runtime
        .observe_authorization(allow_plan(
            revocation.snapshot.clone(),
            fixture.principal,
            fixture.scope,
            [],
        ))
        .expect("post-revocation observation");
    let historical_evidence = fixture
        .runtime
        .observe_authorization(allow_plan(before, fixture.principal, fixture.scope, []))
        .expect("pinned historical observation");

    assert!(before_evidence.paths()[0].matched());
    assert!(historical_evidence.paths()[0].matched());
    // The pinned snapshot keeps its exact root, so its own index answers it
    // after the revocation publishes.
    assert_eq!(historical_evidence.counters().reconstructive_graph_scans, 0);
    assert_eq!(historical_evidence.paths()[0].relations().len(), 2);
    assert!(!after_evidence.paths()[0].matched());
    assert_eq!(after_evidence.paths()[0].adjacency_lists().len(), 2);
}

#[test]
fn foreign_runtime_is_a_typed_denial_before_graph_reads() {
    let source = authorization_fixture();
    let foreign_snapshot = source.runtime.visibility_authority().snapshot();
    let destination = runtime_with_test_schema();
    let denial = destination
        .observe_authorization(allow_plan(
            foreign_snapshot,
            source.principal,
            source.scope,
            [],
        ))
        .expect_err("foreign snapshot must not open destination runtime");

    assert!(matches!(
        denial,
        RelationalAuthorizationObservationDenial::ForeignRuntime { .. }
    ));
}

#[test]
fn empty_observation_plan_is_rejected_before_graph_reads() {
    let fixture = authorization_fixture();
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let denial = RelationalAuthorizationObservationPlan::try_new(
        snapshot,
        fixture.principal,
        fixture.scope,
        ENTITY_KIND,
        ENTITY_KIND,
        [],
        [],
    )
    .expect_err("an observation without graph paths is meaningless");

    assert_eq!(denial, RelationalAuthorizationPlanDenial::NoPaths);
}

#[test]
fn reverse_path_validation_uses_declared_relation_orientation() {
    let fixture = authorization_fixture();
    let snapshot = fixture.runtime.visibility_authority().snapshot();
    let principal_kind = KindId(11);
    let scope_kind = KindId(12);

    let plan = RelationalAuthorizationObservationPlan::try_new(
        snapshot,
        fixture.principal,
        fixture.scope,
        principal_kind,
        scope_kind,
        [RelationalAuthorizationPathPlan::new(
            [RelationalAuthorizationTraversal::new(
                RELATION_KIND,
                scope_kind,
                principal_kind,
                RelationalAuthorizationTraversalDirection::Reverse,
            )],
            [],
        )],
        [],
    );

    assert!(plan.is_ok());
}

#[test]
fn unrelated_graph_scale_does_not_change_observation_work() {
    let fixture = authorization_fixture();
    let baseline_snapshot = fixture.runtime.visibility_authority().snapshot();
    let baseline = fixture
        .runtime
        .observe_authorization(allow_plan(
            baseline_snapshot,
            fixture.principal,
            fixture.scope,
            [],
        ))
        .expect("baseline observation");
    for index in 0..64 {
        let left = create_entity(&fixture.runtime, &format!("unrelated-left-{index}"));
        let right = create_entity(&fixture.runtime, &format!("unrelated-right-{index}"));
        create_relation(
            &fixture.runtime,
            left,
            right,
            &format!("unrelated-edge-{index}"),
        );
    }
    let scaled_snapshot = fixture.runtime.visibility_authority().snapshot();
    let scaled = fixture
        .runtime
        .observe_authorization(allow_plan(
            scaled_snapshot,
            fixture.principal,
            fixture.scope,
            [],
        ))
        .expect("scaled observation");

    assert_eq!(scaled.counters(), baseline.counters());
    assert_eq!(scaled.paths()[0].entities(), baseline.paths()[0].entities());
    assert_eq!(
        scaled.paths()[0].relations(),
        baseline.paths()[0].relations()
    );
}

struct AuthorizationFixture {
    runtime: crate::runtime::RelationalRuntime,
    principal: crate::identity::data::EntityId,
    scope: crate::identity::data::EntityId,
    role_scope_relation: crate::identity::data::RelationId,
}

fn authorization_fixture() -> AuthorizationFixture {
    let mut schema = test_schema_registry();
    let mut unrelated_schema = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    unrelated_schema.relation_kind_id = KindId(99);
    unrelated_schema.relation_kind_name = "unrelated-authorization-relation".to_string();
    let mut unrelated_schema = unrelated_schema.build_registry();
    let unrelated_relation = unrelated_schema.relation_kinds.remove(&KindId(99)).unwrap();
    schema = schema.register_relation_kind(unrelated_relation).unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let principal = create_entity(&runtime, "principal");
    let role = create_entity(&runtime, "approver");
    let scope = create_entity(&runtime, "payment");
    create_relation(&runtime, principal, role, "principal-role");
    let role_scope_relation = create_relation(&runtime, role, scope, "role-scope");
    AuthorizationFixture {
        runtime,
        principal,
        scope,
        role_scope_relation,
    }
}

fn allow_plan(
    snapshot: crate::snapshots::data::SnapshotHandle,
    principal: crate::identity::data::EntityId,
    scope: crate::identity::data::EntityId,
    effects: impl IntoIterator<Item = RelationalAuthorizationEffectTarget>,
) -> RelationalAuthorizationObservationPlan {
    RelationalAuthorizationObservationPlan::try_new(
        snapshot,
        principal,
        scope,
        ENTITY_KIND,
        ENTITY_KIND,
        [allow_path()],
        effects,
    )
    .expect("fixture policy is structurally valid")
}

fn role_and_direct_path_plan(
    snapshot: crate::snapshots::data::SnapshotHandle,
    principal: crate::identity::data::EntityId,
    scope: crate::identity::data::EntityId,
) -> RelationalAuthorizationObservationPlan {
    RelationalAuthorizationObservationPlan::try_new(
        snapshot,
        principal,
        scope,
        ENTITY_KIND,
        ENTITY_KIND,
        [
            allow_path(),
            RelationalAuthorizationPathPlan::new([forward_traversal()], []),
        ],
        [],
    )
    .expect("fixture policy is structurally valid")
}

fn allow_path() -> RelationalAuthorizationPathPlan {
    RelationalAuthorizationPathPlan::new(
        [forward_traversal(), forward_traversal()],
        [RelationalAuthorizationPredicate::new(
            1,
            ENTITY_KIND,
            aspect_field_locator(aspect_key("name"), field_key("name")),
            AspectValue::String(InternedString::Raw("approver".to_string())),
        )],
    )
}

fn forward_traversal() -> RelationalAuthorizationTraversal {
    RelationalAuthorizationTraversal::new(
        RELATION_KIND,
        ENTITY_KIND,
        ENTITY_KIND,
        RelationalAuthorizationTraversalDirection::Forward,
    )
}
