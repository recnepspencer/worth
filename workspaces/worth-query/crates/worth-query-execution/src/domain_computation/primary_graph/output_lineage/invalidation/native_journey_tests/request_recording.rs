//! A request that compares a retained output in full pays to record its row.
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts;

use std::sync::OnceLock;

use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationOutputCorrespondence;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationOutputPosture;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerification;
use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason;
use crate::domain_computation::primary_graph::output_lineage::RetainedOutputCurrentnessRead;
use crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness;
use crate::domain_computation::primary_graph::tests::fixture::publish_relational_mutation;
use crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld;

#[test]
fn a_demand_records_the_row_it_genuinely_verified() {
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
        let mut read = RetainedOutputCurrentnessRead {
            identity: Arc::clone(&identity),
            facts: None,
            consumed_outputs: Arc::from([]),
            native_output_witness: Some(account_witness(&host_custody.borrowed_request_for_test(), &world, runtime, snapshot, entity)),
            consumed_nothing: true,
            verification_requirement: None,
        };
let recorded = || {
            matches!(
                owner.currentness(&positioned, &identity),
                Ok(SourceSettlementCurrentness::Clean)
            )
        };


        assert!(!recorded());
        // Neither absence nor an empty comparison may replace the original
        // own-publication gate. These reads hold no receipt and cannot mark.
        for facts in [None, RetainedSourceFacts::for_test(false, Arc::from([])).for_comparison()] {
            read.facts = facts;

            let stopped = selected.require_current_read(&host_custody.borrowed_request_for_test(),
                runtime, &read, None).unwrap_err();
            assert_eq!(stopped.kind(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Superseded);
            assert!(!recorded(), "factless refusal cannot establish a verified row");
        }
        let label = AccountLabel::reference();
        let locator = graph.layout().field_locator(label.entity(), label.aspect(), label.field()).unwrap().clone();
        let fact = field_fact(&host_custody.borrowed_request_for_test(), runtime, snapshot, entity, locator);
        read.facts = RetainedSourceFacts::for_test(false, Arc::from([fact])).for_comparison();
        assert!(selected
            .require_current_read(&host_custody.borrowed_request_for_test(), runtime, &read, None)
            .is_ok());
        assert!(recorded(), "the genuine native verification registers the row");
    }) }).expect("the fixture read owns its installed host call");
}

#[test]
fn a_verification_records_the_row_it_compared() {
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
            .register_settlement(SettlementRegistration {
                    computation: None,
                    work_membership: None,
                    identity: Arc::clone(&identity),
                    facts: RetainedSourceFacts::for_test(false, Arc::clone(&facts)),
                    output_facts: None,
                    read_basis: positioned.clone(),
                    stale_at_read_basis: OrdSet::new(),
                    requirement: Some(FullVerificationReason::DeclaredChangeUnavailable),
                    upstream: OrdSet::new(),
                })
            .expect("the row requires verification at the selected source");
        let verify = || {
            ConsumedOutputEvidence::verify_at_observation(&host_custody.borrowed_request_for_test(), &identity, &crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::for_test(false, Arc::clone(&facts)).for_comparison().unwrap(), &[], None, &witness, owner, runtime, snapshot, &positioned)
        };
let recorded = || {
            matches!(
                owner.currentness(&positioned, &identity),
                Ok(SourceSettlementCurrentness::Clean)
            )
        };


        assert!(!recorded());
        assert_eq!(
            verify(),
            Ok(ConsumedOutputVerification::Current)
        );
        assert!(recorded(), "the genuine native verification registers the row");
    }) }).expect("the fixture read owns its installed host call");
}

#[test]
fn a_restored_output_registers_before_commit() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let (_, identity) = world
        .application
        .with_host_advancement(|phase| primed_output(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let (head_handle, head) = handle.with_runtime(request_execution, snapshot);
    let restored = ConsumedOutputEvidence::retained_for_test(
        identity,
        Arc::from([]),
        Vec::new(),
        Some(FullVerificationReason::CheckpointRestore),
        Arc::new(head),
    );

    assert_eq!(
        restored.establish_restored_before_commit(request_execution, &handle.source_owner),
        Ok(())
    );

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
    SealedNativeOutputWitness::from_checkpoint_facts(&correspondence, layout, &facts)
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
