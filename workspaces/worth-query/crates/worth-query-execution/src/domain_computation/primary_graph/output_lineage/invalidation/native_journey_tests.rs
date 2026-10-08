//! The ordinary native writer must mark the installed Query actor before its
//! new source head can be read. No World publication or Query feed is involved.

#[path = "native_journey_tests/current_registration.rs"]
mod current_registration;
#[path = "native_journey_tests/dirty_clearance_tests.rs"]
mod dirty_clearance_tests;
#[path = "native_journey_tests/displaced_generation.rs"]
mod displaced_generation;
#[path = "native_journey_tests/equal_field_write.rs"]
mod equal_field_write;
#[path = "native_journey_tests/equality.rs"]
mod equality;
#[cfg(feature = "certification-invalidation-equivalence")]
#[path = "native_journey_tests/full_verification.rs"]
mod full_verification;
#[path = "native_journey_tests/local_requirement.rs"]
mod local_requirement;
#[path = "native_journey_tests/logical_work_scale.rs"]
mod logical_work_scale;
#[path = "native_journey_tests/marking_ceiling.rs"]
mod marking_ceiling;
// Fixture evidence carries no performed output projection, so the
// equivalence oracle correctly refuses it; this proof is about metering.
#[cfg(not(feature = "certification-invalidation-equivalence"))]
#[path = "native_journey_tests/pending_own_evidence.rs"]
mod pending_own_evidence;
#[cfg(not(feature = "certification-invalidation-equivalence"))]
#[path = "native_journey_tests/precommit_chain.rs"]
mod precommit_chain;
#[path = "native_journey_tests/replay_propagation.rs"]
mod replay_propagation;
#[path = "native_journey_tests/required_hints.rs"]
mod required_hints;
#[path = "native_journey_tests/retained_categories.rs"]
mod retained_categories;
#[path = "native_journey_tests/shared_versions.rs"]
mod shared_versions;
#[path = "native_journey_tests/unchanged_history.rs"]
mod unchanged_history;
#[path = "native_journey_tests/undeclared_change.rs"]
mod undeclared_change;
#[path = "native_journey_tests/verified_current.rs"]
mod verified_current;

use std::{any::TypeId, collections::BTreeMap, sync::Arc};

use im::OrdSet;
use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, InternedString, LocatorAuthority,
};
use worth_relational::facade::{
    identity::EntityId,
    mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent},
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    snapshots::SnapshotHandle,
    transactions::{
        AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};

use super::super::{ProductCoordinate, RecordedSettlementIdentity, SemanticSource};
use super::{SettlementRegistration, SourceInvalidationOwner, SourceSettlementCurrentness};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    tests::fixture::{
        installed_authorization_world, live_scope, release_test_commit_snapshot, AccountLabel,
        AccountStatus,
    },
    WorthQueryPrincipalResolutionMode,
};

struct StatusOutput;
struct LabelOutput;
struct DownstreamOutput;
struct LateOutput;

fn snapshot(runtime: &RelationalRuntime) -> (SnapshotHandle, PositionedRelationalSnapshot) {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let handle = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let proof = runtime.read_truth().positioned_snapshot(&handle).unwrap();
    (handle, proof)
}

fn write_field(
    runtime: &mut RelationalRuntime,
    entity: EntityId,
    locator: AspectFieldLocator,
    value: &str,
) {
    let committed = write_batch(
        runtime,
        WorkerIntentBatch::new("native-invalidation-journey").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    locator,
                    AspectValue::String(InternedString::Raw(value.to_owned())),
                )])),
            }),
        )),
    );
    release_test_commit_snapshot(runtime, &committed);
}

fn write_batch(
    runtime: &mut RelationalRuntime,
    batch: WorkerIntentBatch,
) -> worth_relational::facade::transactions::CommitResult {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction.push_batch(batch).unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    let RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("ordinary native field mutation must publish");
    };
    runtime.settle_performed_publication(performed).unwrap()
}

fn field_fact(
    runtime: &RelationalRuntime,
    handle: &SnapshotHandle,
    entity: EntityId,
    locator: AspectFieldLocator,
) -> WorthQueryApplicationObservedFact {
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    );
    let revision = runtime
        .read_truth()
        .project_snapshot(handle)
        .unwrap()
        .entity_field_revision(entity, &locator)
        .unwrap();
    WorthQueryApplicationObservedFact::SourceFieldRevision {
        entity_id: entity,
        locator,
        native_revision: Some(revision),
    }
}

fn register(
    owner: &SourceInvalidationOwner,
    identity: Arc<RecordedSettlementIdentity>,
    facts: Arc<[WorthQueryApplicationObservedFact]>,
    basis: &PositionedRelationalSnapshot,
    upstream: OrdSet<Arc<RecordedSettlementIdentity>>,
) {
    owner
        .register_settlement(
            SettlementRegistration {
                work_membership: None,
                identity,
                facts,
                output_facts: None,
                read_basis: basis.clone(),
                stale_at_read_basis: im::OrdSet::new(),
                requirement: None,
                upstream,
            },
            &mut owner.edit_admission(),
        )
        .expect("native aligned settlement registers");
}

fn currentness(
    owner: &SourceInvalidationOwner,
    basis: &PositionedRelationalSnapshot,
    identity: &RecordedSettlementIdentity,
) -> SourceSettlementCurrentness {
    owner
        .currentness(basis, identity, &mut owner.edit_admission())
        .unwrap()
}

#[test]
fn ordinary_native_writer_marks_only_matched_fields_and_actual_downstream_edges() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let source = |output_binding| {
        SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity().clone(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding,
    }
    };
    let a =
        RecordedSettlementIdentity::retain(&source(TypeId::of::<StatusOutput>()), coordinate, 0);
    let b = RecordedSettlementIdentity::retain(&source(TypeId::of::<LabelOutput>()), coordinate, 0);
    let c = RecordedSettlementIdentity::retain(
        &source(TypeId::of::<DownstreamOutput>()),
        coordinate,
        0,
    );
    let late =
        RecordedSettlementIdentity::retain(&source(TypeId::of::<LateOutput>()), coordinate, 0);
    handle.with_runtime_mut(|runtime| {
        // Ensure the real companion has a selected cell even if fixture
        // installation happened before its subscription became active.
        write_field(runtime, entity, label.clone(), "prime");
        let (before_handle, before) = snapshot(runtime);
        let status_facts: Arc<[_]> =
            Arc::from([field_fact(runtime, &before_handle, entity, status.clone())]);
        register(
            owner,
            a.clone(),
            status_facts.clone(),
            &before,
            OrdSet::new(),
        );
        register(
            owner,
            b.clone(),
            Arc::from([field_fact(runtime, &before_handle, entity, label.clone())]),
            &before,
            OrdSet::new(),
        );
        register(
            owner,
            c.clone(),
            Arc::from([]),
            &before,
            OrdSet::unit(a.clone()),
        );
        assert!(matches!(
            currentness(owner, &before, &a),
            SourceSettlementCurrentness::Clean
        ));
        write_field(runtime, entity, label, "changed-label");
        let (label_handle, label_basis) = snapshot(runtime);
        assert!(
            matches!(
                currentness(owner, &label_basis, &a),
                SourceSettlementCurrentness::Clean
            ),
            "the sibling aspect touch must not dirty a field-local settlement"
        );
        assert!(matches!(
            currentness(owner, &label_basis, &c),
            SourceSettlementCurrentness::Clean
        ));
        match currentness(owner, &label_basis, &b) {
            SourceSettlementCurrentness::Dirty(ordinals) => assert_eq!(ordinals, OrdSet::unit(0)),
            _ => panic!("the matching label fact must be dirty before its head is read"),
        }
        assert!(
            matches!(
                currentness(owner, &before, &b),
                SourceSettlementCurrentness::Clean
            ),
            "newer marks cannot leak into the retained earlier snapshot"
        );
        write_field(runtime, entity, status, "closed");
        let (after_handle, after) = snapshot(runtime);
        assert!(matches!(
            currentness(owner, &after, &a),
            SourceSettlementCurrentness::Dirty(_)
        ));
        match currentness(owner, &after, &c) {
            SourceSettlementCurrentness::PendingUpstream(upstream) => {
                assert_eq!(upstream, OrdSet::unit(a))
            }
            _ => panic!("only the actual downstream edge must be pending"),
        }
        register(owner, late.clone(), status_facts, &before, OrdSet::new());
        assert!(
            matches!(
                currentness(owner, &after, &late),
                SourceSettlementCurrentness::Dirty(_)
            ),
            "late registration must replay actual intervening deliveries"
        );
        for snapshot in [before_handle, label_handle, after_handle] {
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        }
    });
}
