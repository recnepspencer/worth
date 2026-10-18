//! A request that compares a retained output in full pays to record its row.
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts;

use std::sync::OnceLock;

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
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let (entity, identity) = world
        .application
        .with_host_advancement(|phase| primed_output(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); handle.with_runtime(request_execution, |runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let positioned = runtime.read_truth().positioned_snapshot(snapshot).unwrap();
        let read = RetainedOutputCurrentnessRead {
            _computation: crate::domain_computation::primary_graph::output_lineage::ComputationSourceEvidence::for_test(false).certify_current().unwrap(),
            identity: Arc::clone(&identity),
            facts: crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, Arc::from([])).for_comparison().unwrap(),
            native_output_witness: Some(account_witness(&host_custody.borrowed_request_for_test(), &world, runtime, snapshot, entity)),
            consumed_nothing: true,
            verification_requirement: None,
        };
        let recorded = || {
            matches!(
                owner.currentness(&positioned, &identity, &mut owner.edit_admission()),
                Ok(SourceSettlementCurrentness::Clean)
            )
        };
        assert!(!recorded());
        // Search the smallest explicit caller restriction on the same row;
        // the installed default has no finite upper bound.
        let least = (1usize..)
            .find(|&work| {
                let mut request = owner.edit_admission_with_work_limit(Some(work));
                selected
                    .require_current_read(&host_custody.borrowed_request_for_test(), runtime, &read, &mut request)
                    .is_ok()
            })
            .expect("some meter verifies the output");
        assert!(
            !recorded(),
            "the least meter that answers ({least}) has nothing left to record the row"
        );
        assert!(selected
            .require_current_read(&host_custody.borrowed_request_for_test(), runtime, &read, &mut owner.edit_admission())
            .is_ok());
        assert!(recorded(), "a meter with room records the verified row");
    }) }).expect("the fixture read owns its installed host call");
}

#[test]
fn a_verification_records_the_row_it_compared_on_its_own_meter() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let (entity, identity) = world
        .application
        .with_host_advancement(|phase| primed_output(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); handle.with_runtime(request_execution, |runtime| {
        let snapshot = selected.application_basis().snapshot_handle();
        let positioned = runtime.read_truth().positioned_snapshot(snapshot).unwrap();
        let witness = account_witness(&host_custody.borrowed_request_for_test(), &world, runtime, snapshot, entity);
        let facts: Arc<[WorthQueryApplicationObservedFact]> = Arc::from([]);
        owner
            .register_settlement(
                SettlementRegistration {
                    work_membership: None,
                    identity: Arc::clone(&identity),
                    facts: RetainedSourceFacts::for_test(false, Arc::clone(&facts)),
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
            ConsumedOutputEvidence::verify_at_observation(&host_custody.borrowed_request_for_test(),
                &identity,
                &crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, Arc::clone(&facts)).for_comparison().unwrap(),
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
        // Search the smallest explicit caller restriction on the same row;
        // the installed default has no finite upper bound.
        let least = (1usize..)
            .find(|&work| {
                let mut request = owner.edit_admission_with_work_limit(Some(work));
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
    }) }).expect("the fixture read owns its installed host call");
}

#[test]
fn a_commit_meter_that_cannot_record_a_restored_output_stops_the_commit() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let (_, identity) = world
        .application
        .with_host_advancement(|phase| primed_output(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let (head_handle, head) = handle.with_runtime(request_execution, snapshot);
    let restored = ConsumedOutputEvidence::retained_for_test(
        owner,
        identity,
        Arc::from([]),
        Vec::new(),
        Some(FullVerificationReason::CheckpointRestore),
        Arc::new(head),
    );
    let mut exhausted = owner.edit_admission_with_work_limit(Some(1));
    exhausted.charge_external_work(1).unwrap();
    assert_eq!(
        restored.establish_restored_before_commit(
            request_execution,
            &handle.source_owner,
            &mut exhausted
        ),
        Err(ConsumedOutputVerificationStop::WorkExhausted),
        "the commit's answer reads the row, so its meter pays to record it"
    );
    let mut request = owner.edit_admission();
    assert_eq!(
        restored.establish_restored_before_commit(
            request_execution,
            &handle.source_owner,
            &mut request
        ),
        Ok(())
    );
    assert!(request.charged_work() > 0);
    handle.with_runtime_mut(request_execution, |runtime| {
        runtime.snapshots().release_snapshot(&head_handle).unwrap();
    });
}

/// The account's sealed native output witness at `snapshot`.
pub(super) fn account_witness(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole {
            role: "account".to_owned(),
            posture: WorthQueryApplicationOutputPosture::Preserve,
            entity_name: "Account".to_owned(),
            entity,
        }],
        |_| Some(TypeId::of::<()>()),
        execution,
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
fn primed_output(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
) -> (EntityId, Arc<RecordedSettlementIdentity>) {
    let entity = world
        .selected_product(&(phase_custody).test_host_request())
        .resolve_entity(
            phase_custody,
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
        &phase_custody.borrowed_request_for_test(),
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
    let (_, product, _) = world
        .selected_product(&(phase_custody).test_host_request())
        .into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: OutputBindingIdentity::declared("LabelOutput"),
    };
    (
        entity,
        RecordedSettlementIdentity::retain(&source, coordinate, 0),
    )
}
