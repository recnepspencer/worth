//! A request that compares a retained output in full pays to record its row.

use std::{num::NonZeroUsize, sync::OnceLock};

use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputPosture,
        WorthQueryCheckpointOutputRole,
    },
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    },
    output_lineage::{
        invalidation::{FullVerificationReason, InvalidationEditAdmission},
        RetainedOutputCurrentnessRead, SealedNativeOutputWitness,
    },
    tests::fixture::{publish_relational_mutation, AuthorizationWorld},
};

#[test]
fn a_demand_records_the_row_it_verified_on_its_own_meter() {
    let world = installed_authorization_world(true);
    let (entity, identity) = primed_output(&world);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let selected = world.selected_product();
    handle.with_runtime(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let positioned = runtime.read_truth().positioned_snapshot(snapshot).unwrap();
        let read = RetainedOutputCurrentnessRead {
            identity: Arc::clone(&identity),
            facts: Arc::from([]),
            native_output_witness: Some(account_witness(&world, runtime, snapshot, entity)),
            consumed_nothing: true,
            work: 0,
        };
        let recorded = || {
            matches!(
                owner.currentness(&positioned, &identity, &mut owner.edit_admission()),
                Ok(SourceSettlementCurrentness::Clean)
            )
        };
        assert!(!recorded());
        let least = (1..=owner.edit_admission().remaining_work())
            .find(|&work| {
                let mut request = owner.edit_admission_within(NonZeroUsize::new(work).unwrap());
                selected
                    .require_current_read(runtime, &read, &mut request)
                    .is_ok()
            })
            .expect("some meter verifies the output");
        assert!(
            !recorded(),
            "the least meter that answers ({least}) has nothing left to record the row"
        );
        assert!(selected
            .require_current_read(runtime, &read, &mut owner.edit_admission())
            .is_ok());
        assert!(recorded(), "a meter with room records the verified row");
    });
}

#[test]
fn a_verification_records_the_row_it_compared_on_its_own_meter() {
    let world = installed_authorization_world(true);
    let (entity, identity) = primed_output(&world);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let selected = world.selected_product();
    handle.with_runtime(|runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let positioned = runtime.read_truth().positioned_snapshot(snapshot).unwrap();
        let witness = account_witness(&world, runtime, snapshot, entity);
        let facts: Arc<[WorthQueryApplicationObservedFact]> = Arc::from([]);
        owner
            .register_settlement(
                SettlementRegistration {
                    work_membership: None,
                    identity: Arc::clone(&identity),
                    facts: Arc::clone(&facts),
                    output_facts: None,
                    read_basis: positioned.clone(),
                    stale_at_read_basis: OrdSet::new(),
                    requirement: Some(FullVerificationReason::DeclaredChangeUnavailable),
                    upstream: OrdSet::new(),
                },
                &mut owner.edit_admission(),
            )
            .expect("the row requires verification at the selected source");
        let verify = |admission: &mut InvalidationEditAdmission| {
            ConsumedOutputEvidence::verify_at_observation(
                &identity,
                &facts,
                &[],
                None,
                &witness,
                owner,
                runtime,
                snapshot,
                &positioned,
                admission,
            )
        };
        let recorded = || {
            matches!(
                owner.currentness(&positioned, &identity, &mut owner.edit_admission()),
                Ok(SourceSettlementCurrentness::Clean)
            )
        };
        assert!(!recorded());
        let least = (1..=owner.edit_admission().remaining_work())
            .find(|&work| {
                let mut request = owner.edit_admission_within(NonZeroUsize::new(work).unwrap());
                verify(&mut request) == Ok(ConsumedOutputVerification::Current)
            })
            .expect("some meter verifies the output");
        assert!(
            !recorded(),
            "the least meter that answers ({least}) has nothing left to record the row"
        );
        assert_eq!(
            verify(&mut owner.edit_admission()),
            Ok(ConsumedOutputVerification::Current)
        );
        assert!(recorded(), "a meter with room records the verified row");
    });
}

#[test]
fn a_commit_meter_that_cannot_record_a_restored_output_stops_the_commit() {
    let world = installed_authorization_world(true);
    let (_, identity) = primed_output(&world);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let (head_handle, head) = handle.with_runtime(snapshot);
    let restored = ConsumedOutputEvidence::retained_for_test(
        owner,
        identity,
        Arc::from([]),
        Vec::new(),
        Some(FullVerificationReason::CheckpointRestore),
        Arc::new(head),
    );
    let mut exhausted = owner.edit_admission();
    exhausted
        .charge_external_work(u64::try_from(exhausted.remaining_work()).unwrap())
        .unwrap();
    assert_eq!(
        restored.establish_restored_before_commit(&handle.source_owner, &mut exhausted),
        Err(ConsumedOutputVerificationStop::WorkExhausted),
        "the commit's answer reads the row, so its meter pays to record it"
    );
    let mut request = owner.edit_admission();
    assert_eq!(
        restored.establish_restored_before_commit(&handle.source_owner, &mut request),
        Ok(())
    );
    assert!(request.charged_work() > 0);
    handle.with_runtime_mut(|runtime| {
        runtime.snapshots().release_snapshot(&head_handle).unwrap();
    });
}

/// The account's sealed native output witness at `snapshot`.
fn account_witness(
    world: &AuthorizationWorld,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
) -> Arc<OnceLock<SealedNativeOutputWitness>> {
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph.layout();
    let owner = &graph.integration_handle().source_owner.invalidation_owner;
    let truth = runtime.read_truth();
    let mut facts = vec![WorthQueryApplicationObservedFact::Entity {
        entity_id: entity,
        kind: truth
            .exact_snapshot_live_entity_kind(snapshot, entity)
            .unwrap(),
    }];
    for aspect in layout.native_output_aspects("Account") {
        facts.push(WorthQueryApplicationObservedFact::SourceAspectRevision {
            entity_id: entity,
            aspect: aspect.clone(),
            native_revision: truth
                .exact_snapshot_entity_aspect_version(snapshot, entity, aspect)
                .unwrap(),
        });
    }
    let correspondence = WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
        TypeId::of::<()>(),
        TypeId::of::<()>(),
        std::collections::BTreeSet::new(),
        vec![WorthQueryCheckpointOutputRole {
            role: "account".to_owned(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "Account".to_owned(),
            entity,
        }],
        |_| Some(TypeId::of::<()>()),
    )
    .unwrap();
    SealedNativeOutputWitness::from_checkpoint_facts(
        &correspondence,
        layout,
        &facts,
        owner,
        &mut owner.edit_admission(),
    )
    .unwrap()
    .expect("complete native facts reconstruct the witness")
}

/// Publishes through the World so the companion has a selected cell at the
/// head, and names one label output of the open account.
fn primed_output(world: &AuthorizationWorld) -> (EntityId, Arc<RecordedSettlementIdentity>) {
    let entity = world
        .selected_product()
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    publish_relational_mutation(
        world,
        WorkerIntentBatch::new("request-recording-prime").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    label,
                    AspectValue::String(InternedString::Raw("prime".to_owned())),
                )])),
            }),
        )),
    );
    let (_, product, _) = world.selected_product().into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: TypeId::of::<LabelOutput>(),
    };
    (
        entity,
        RecordedSettlementIdentity::retain(&source, coordinate, 0),
    )
}
