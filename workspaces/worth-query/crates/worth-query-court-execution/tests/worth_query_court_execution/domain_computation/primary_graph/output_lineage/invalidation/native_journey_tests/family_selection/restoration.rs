//! Courts 6, 7 and 8 (public #429, S1 STOP 4): a producer head that arrives
//! without a wake, through `record_restoration` or a republication whose
//! registration refused, leaves a `Missing` reader's marks unable to certify
//! it. The reader's settlement carries its
//! `FamilySelection` fact on the world's invalidation owner; the head is
//! recorded on the world's lineage through the production recorder.
use super::{certify, install_family, read_family, retained, Fact, FactMovement, Kind};
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryCheckpointOutputRole,
};
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::identity_kind::WorthQueryCheckpointSourceIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerification;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::publication::register_republished;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::SealedNativeOutputWitness;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_source_identity::RecordedSourceIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::{
    CertificationBasis, WorthQueryApplicationOutputLineage,
};
use worth_query_execution::facade::primary_graph::settlement_registrations_on_this_thread_for_test as registrations;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionOutputs;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;

/// A restored row's retained facts: the producer read only its scope entity.
fn entity_facts(entity: EntityId) -> RetainedSourceFacts {
    retained_facts(vec![WorthQueryApplicationObservedFact::SourceEntity { entity_id: entity }])
}

fn retained_facts(facts: Vec<WorthQueryApplicationObservedFact>) -> RetainedSourceFacts {
    ComputationSourceEvidence::for_test(false).retain_facts_for_test(Arc::from(facts))
}

/// The producer's checkpoint correspondence: its one `retained-account` role
/// preserves the open account.
fn checkpoint_correspondence(
    entity: EntityId,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryApplicationOutputCorrespondence {
    WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
        TypeId::of_for_test::<RetentionOutputs>(),
        TypeId::of_for_test::<RetentionOutputs>(),
        std::iter::empty(),
        &[WorthQueryCheckpointOutputRole::new_for_test(
            "retained-account".to_owned(),
            ApplicationMutationOutputPosture::Preserve,
            "Account".to_owned(),
            entity,
        )],
        |_| Some(TypeId::of_for_test::<Account>()),
        execution,
    )
    .unwrap()
}

/// The producer's checkpoint facts: its scope entity, and the open account's
/// kind and every native output aspect revision at `snapshot`, which a
/// restored-root reader rebuilds the head's witness from.
fn checkpoint_facts(
    world: &AuthorizationWorld,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
) -> Vec<WorthQueryApplicationObservedFact> {
    let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
    let layout = graph.layout_for_test();
    let truth = runtime.read_truth();
    let mut facts = vec![
        WorthQueryApplicationObservedFact::SourceEntity { entity_id: entity },
        WorthQueryApplicationObservedFact::Entity {
            entity_id: entity,
            kind: truth.read_exact_snapshot_live_entity_kind(execution, snapshot, entity).unwrap().unwrap(),
        },
    ];
    for aspect in layout.native_output_aspects_for_test("Account") {
        facts.push(WorthQueryApplicationObservedFact::SourceAspectRevision {
            entity_id: entity,
            aspect: aspect.clone(),
            native_revision: truth
                .read_exact_snapshot_entity_aspect_version(execution, snapshot, entity, aspect)
                .unwrap()
                .unwrap(),
        });
    }
    facts
}

/// What the courts observe after the head arrived. The head is an
/// unwitnessed checkpoint root, so a reader rebuilds its witness from its
/// checkpoint facts and compares them in full.
struct Restored<'w> {
    world: &'w AuthorizationWorld,
    owner: &'w SourceInvalidationOwner,
    reader: Arc<RecordedSettlementIdentity>,
    /// The reader's recorded `FamilySelection`, which selected `Missing`.
    selection: Fact,
    /// The positioned snapshot the reader registered at.
    basis: PositionedRelationalSnapshot,
}

/// How the producer's head arrives without waking the reader.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arrival {
    /// A checkpoint restoration, which registers no settlement.
    Restored,
    /// A republication whose `register_republished` refuses after the head
    /// is recorded, as production's does for a predecessor the owner never
    /// registered; the head keeps the refusal as its requirement.
    RefusedRepublication,
}

/// Read the family of the open account (`Missing`), register the reader's
/// settlement with that fact, then record the producer's head through
/// `record_restoration`. No settlement registration runs for the head, so
/// nothing wakes the reader.
fn missing_reader_then_restored_head(arrival: Arrival, test: impl FnOnce(&Restored<'_>)) {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    install_family(&world, vec![retained()]);
    let (kind, facts) = read_family(&world, "open");
    assert_eq!(kind, Kind::Missing, "the family has no head before the producer completes");
    let [selection @ Fact::FamilySelection { .. }] = facts.as_slice() else {
        panic!("the Missing read must record exactly one FamilySelection fact: {facts:?}");
    };
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts_for_test();
    let observation = product.observation_for_test();
    let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
    let authority = (*world.application.runtime_for_test()).authority_identity_for_test().as_u64();
    let schema = (*world.application.installed_schema_for_test()).binding_identity();
    let scope =
        worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity);

    // Settlement identities of real recorded rows the world's owner has not
    // registered: the reader's, before it registers, and a republication's
    // predecessor.
    let mut scratch = WorthQueryApplicationOutputLineage::default();
    scratch.fixture_binding(TypeId::of_for_test::<StatusOutput>(), "StatusOutput");
    let mut scratch_identity = |byte: u8| {
        scratch
            .record_restoration_for_test(
                TypeId::of_for_test::<StatusOutput>(),
                authority,
                schema.clone(),
                scope,
                observation,
                Arc::new(WorthQueryApplicationOutputCorrespondence::default()),
                RecordedSourceIdentity::Checkpoint(WorthQueryCheckpointSourceIdentity::new_for_test([byte; 32])),
                [byte + 1; 32],
                None,
                [byte + 2; 32],
                entity_facts(entity),
                None,
                None,
                None,
            )
            .expect("the scratch row is recorded")
            .identity
    };
    let reader = scratch_identity(0x61);
    let predecessor = scratch_identity(0x64);
    let (snapshot_handle, basis) = handle.with_runtime(execution, snapshot);
    let head_facts =
        handle.with_runtime(execution, |runtime| checkpoint_facts(&world, execution, runtime, &snapshot_handle, entity));
    register(owner, Arc::clone(&reader), Arc::from([selection.clone()]), &basis, OrdSet::new());

    let before = registrations();
    let correspondence = checkpoint_correspondence(entity, execution);
    let witness = SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, graph.layout_for_test(), &head_facts)
        .unwrap()
        .expect("the head's checkpoint facts cover its output");
    let head = handle
        .output_lineage
        .lock()
        .unwrap()
        .record_restoration_for_test(
            TypeId::of::<RetentionOutputs>(),
            authority,
            schema,
            scope,
            observation,
            Arc::new(correspondence),
            RecordedSourceIdentity::Checkpoint(WorthQueryCheckpointSourceIdentity::new_for_test([0x71; 32])),
            [0x72; 32],
            None,
            [0x73; 32],
            retained_facts(head_facts),
            // A republished row carries its sealed witness; a restored one
            // proves its root from its checkpoint facts instead.
            (arrival == Arrival::RefusedRepublication).then(|| Arc::clone(&witness)),
            None,
            None,
        )
        .expect("the producer's head is recorded");
    if arrival == Arrival::RefusedRepublication {
        let refused = register_republished(
            owner,
            &predecessor,
            Arc::clone(&head.identity),
            head.facts,
            &[],
            witness.get().unwrap(),
            basis.clone(),
        );
        assert_eq!(
            refused,
            Err(FullVerificationReason::MissingSettlement),
            "a predecessor the owner never registered refuses the republication's registration"
        );
        handle
            .output_lineage
            .lock()
            .unwrap()
            .require_settlement_verification_for_test(&head.identity, FullVerificationReason::MissingSettlement);
    }
    assert_eq!(registrations(), before, "the head registers no settlement, so nothing woke the reader");

    test(&Restored { world: &world, owner, reader, selection: selection.clone(), basis });
    handle.with_runtime_for_test(execution, |runtime| runtime.snapshots().release_snapshot(&snapshot_handle).unwrap());
}

/// Court 6: the reader's marks never certify its `FamilySelection` fact, so
/// a head that arrived without a wake still leaves the reader requiring full
/// verification. Marks that keyed the fact Clean would answer `Clean` here.
#[test]
fn a_family_selection_keyed_clean_is_caught() {
    missing_reader_then_restored_head(Arrival::Restored, |restored| {
        assert_eq!(marks(restored), "FullVerificationRequired", "marks never certify a FamilySelection fact");
    });
}

/// The owner's marks answer for the reader. `FullVerificationRequired` is
/// asserted to carry the fact's own `UnsupportedFact`.
fn marks(restored: &Restored<'_>) -> String {
    match restored.owner.currentness_for_test(&restored.basis, &restored.reader).unwrap() {
        SourceSettlementCurrentness::Clean => "Clean".to_owned(),
        SourceSettlementCurrentness::Dirty(_) => "Dirty".to_owned(),
        SourceSettlementCurrentness::PendingUpstream(_) => "PendingUpstream".to_owned(),
        SourceSettlementCurrentness::FullVerificationRequired(reason) => {
            assert_eq!(reason, FullVerificationReason::UnsupportedFact, "the reason is the fact's own");
            "FullVerificationRequired".to_owned()
        }
        SourceSettlementCurrentness::Foreign => "Foreign".to_owned(),
    }
}

/// Court 7: after the restored head, the reader's verification compares its
/// selection in full and finds it moved, as a fresh read selects `Unique`.
#[test]
fn a_restoration_settle_reads_the_current_selection() {
    missing_reader_then_restored_head(Arrival::Restored, settles_to_the_current_selection);
}

/// Court 8: a republication whose registration refused published the head
/// without a wake. The reader's marks still require full verification, and
/// its verification re-resolves the selection, finds it moved, and so
/// executes.
#[test]
fn a_failed_republication_registration_reads_correctly() {
    missing_reader_then_restored_head(Arrival::RefusedRepublication, |restored| {
        assert_eq!(marks(restored), "FullVerificationRequired", "marks never certify a FamilySelection fact");
        settles_to_the_current_selection(restored);
    });
}

/// A fresh read selects the head, the recorded `Missing` selection is
/// `Moved` at the basis, and the reader's verification compares it in full.
fn settles_to_the_current_selection(restored: &Restored<'_>) {
    let (fresh, _) = read_family(restored.world, "open");
    assert_eq!(fresh, Kind::Unique, "a fresh read selects the restored head");
    assert_eq!(
        certify(restored.world, &restored.selection),
        FactMovement::from_equal(false),
        "the recorded Missing selection is not current at the basis"
    );
    let world = restored.world;
    let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
    let handle = graph.integration_handle_for_test();
    let facts = RetainedSourceFacts::for_test_for_test(false, Arc::from([restored.selection.clone()]));
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let serial = worth_execution::ExecutionRequest::serial(&host_request);
    let verification = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let selected = world.selected_product(&host);
            handle.with_runtime(&serial, |runtime| {
                let snapshot = selected.application_basis_for_test().snapshot_handle();
                let positioned = runtime.read_truth().positioned_snapshot(snapshot).unwrap();
                let product = selected.product();
                ConsumedOutputEvidence::verify_candidate_with_admission_for_test(
                    &host.borrowed_request_for_test(),
                    &restored.reader,
                    &facts.for_comparison().unwrap(),
                    &[],
                    None,
                    &restored.basis,
                    restored.owner,
                    runtime,
                    CertificationBasis::of(product.lifecycle_incarnation(), product.reference_generation(), snapshot),
                    &positioned,
                )
            })
        })
        .expect("the fixture verification owns its installed host call");
    assert_eq!(
        verification,
        Ok(ConsumedOutputVerification::ChangedDirectFact(0)),
        "the reader's verification compares its selection in full and finds it moved"
    );
}
