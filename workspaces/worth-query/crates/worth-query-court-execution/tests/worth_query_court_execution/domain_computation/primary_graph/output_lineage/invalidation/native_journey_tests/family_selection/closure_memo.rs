//! Court 14 (h), public #429: the closure memo across an image refresh.
//! Two readers, `P1` and `P2`, each record the family's `Unique` selection
//! as their one fact. A verification walk that clears a dirty-but-unchanged
//! sibling `S` between them moves the root payload without changing what
//! any compared row means, so `P2` still certifies `Unmoved` and takes the
//! closure `P1`'s re-resolution memoized. An equal link recorded inside the
//! frame, or a re-registration of a row the memo read, clears it, and the
//! settlement rows compared show the re-comparison.
use super::{install_family, read_family, retained, Fact, FactMovement, Kind};
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::equality::register_alias;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerification;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::computation_changes::ComputationRegistration;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::verification::DirtyReverification;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::{
    CertificationBasis, CertificationScope,
};
use worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding;
use worth_query_execution::facade::primary_graph::family_selection_cost_on_this_thread_for_test as cost;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::own_write_fixture::with_first_completion;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::{
    AccountBlocked, PrincipalIdentityField,
};
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    CreateIntent, CreatedRelationRef, DeleteRelationIntent, EntityReference, RelationMutationIntent, RelationSpec,
};

/// The world after the producer's first completion, with two dirty
/// siblings whose relation was added and removed again, and the readers'
/// recorded selection.
struct Readers<'w> {
    world: &'w AuthorizationWorld,
    owner: &'w SourceInvalidationOwner,
    selection: Fact,
    readers: [ConsumedOutputEvidence; 2],
    /// Dirty-but-unchanged siblings: the walk clears the first, the memo
    /// variant the second.
    siblings: [ConsumedOutputEvidence; 2],
    account: EntityId,
    coordinate: ProductCoordinate,
}

fn identity(world: &AuthorizationWorld, coordinate: ProductCoordinate, account: EntityId, name: &'static str) -> Arc<RecordedSettlementIdentity> {
    RecordedSettlementIdentity::retain_for_test(
        &SemanticSource::new_for_test(
            (*world.application.runtime_for_test()).authority_identity_for_test().as_u64(),
            (*world.application.installed_schema_for_test()).binding_identity().clone(),
            WorthQueryOperationScopeEntityBinding::from_entity_for_test(account),
            OutputBindingIdentity::declared_for_test(name),
        ),
        coordinate,
        0,
    )
}

/// Register the readers and the siblings and read the family, then run `test`.
fn readers(test: impl FnOnce(&Readers<'_>)) {
    with_first_completion(
        |world| install_family(world, vec![retained()]),
        |world, _receipt, ()| {
            let _observation =
                worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
            let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
            let execution = &worth_execution::ExecutionRequest::serial(&host_request);
            let (account, principal) = world
                .application
                .with_host_advancement(|phase| {
                    let selected = world.selected_product(&phase.test_host_request_for_test());
                    let account = selected
                        .resolve_entity(&phase, AccountStatus::at("AccountStatus"), "1".to_owned(), &live_scope(), WorthQueryPrincipalResolutionMode::Ordinary)
                        .unwrap()
                        .entity_id_for_test();
                    let principal = selected
                        .resolve_entity(&phase, PrincipalIdentityField::at("PrincipalIdentityField"), 1, &live_scope(), WorthQueryPrincipalResolutionMode::Ordinary)
                        .unwrap()
                        .entity_id_for_test();
                    (account, principal)
                })
                .expect("the fixture read owns its installed host call");
            let observation = resolve_observation(world);
            let coordinate = ProductCoordinate::new_for_test(observation.0, observation.1);
            let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
            let handle = graph.integration_handle_for_test();
            let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
            let kind = *graph.layout_for_test().relation(AccountBlocked::reference().name()).unwrap().kind_for_test();

            // Two siblings observe that no blocking relation exists; adding one
            // and removing it again leaves them dirty and unchanged.
            let sibling_facts: Arc<[WorthQueryApplicationObservedFact]> = Arc::from([WorthQueryApplicationObservedFact::Relation {
                relation_kind: kind,
                from: principal,
                to: account,
                matching_relations: Vec::new(),
            }]);
            let (sibling_handle, sibling_basis) = handle.with_runtime(execution, snapshot);
            let sibling_ids = ["ClosureMemoSiblingA", "ClosureMemoSiblingB"].map(|name| identity(world, coordinate, account, name));
            for sibling in &sibling_ids {
                owner
                    .register_settlement_for_test(SettlementRegistration::new_for_test(
                        Some(ComputationRegistration::retained_for_test()),
                        None,
                        Arc::clone(sibling),
                        RetainedSourceFacts::for_test_for_test(false, Arc::clone(&sibling_facts)),
                        None,
                        sibling_basis.clone(),
                        OrdSet::new(),
                        None,
                        OrdSet::new(),
                    ))
                    .unwrap();
            }
            handle.with_runtime_for_test(execution, |runtime| {
                let created = CreatedRelationRef {
                    partition_id: PartitionId::main(),
                    kind_id: kind,
                    client_key: ClientKey::raw("closure-memo-relation"),
                    source: EntityReference::Existing(principal),
                    target: EntityReference::Existing(account),
                };
                let added = write_batch(
                    execution,
                    runtime,
                    WorkerIntentBatch::new("add-closure-memo-relation").push(MutationIntent::Create(CreateIntent::Relation(
                        RelationSpec {
                            partition_id: created.partition_id,
                            kind_id: created.kind_id,
                            client_key: created.client_key.clone(),
                            source: created.source.clone(),
                            target: created.target.clone(),
                            fields: AspectFieldPatch::default(),
                        },
                    ))),
                );
                let relation = added.created_relation(&created).unwrap();
                release_test_commit_snapshot(runtime, &added);
                let removed = write_batch(
                    execution,
                    runtime,
                    WorkerIntentBatch::new("remove-closure-memo-relation").push(MutationIntent::Relation(
                        RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: relation }),
                    )),
                );
                release_test_commit_snapshot(runtime, &removed);
                runtime.snapshots().release_snapshot(&sibling_handle).unwrap();
            });

            let (kind, facts) = read_family(world, "1");
            assert_eq!(kind, Kind::Unique, "the first completion's head is the family's one member");
            let [selection @ Fact::FamilySelection { .. }] = facts.as_slice() else {
                panic!("the read must record exactly one FamilySelection fact: {facts:?}");
            };
            let (read_handle, read_basis) = handle.with_runtime(execution, snapshot);
            for (name, dirty) in [("ClosureMemoSiblingA", true), ("ClosureMemoSiblingB", true)] {
                let sibling = identity(world, coordinate, account, name);
                assert_eq!(
                    matches!(currentness(owner, &read_basis, &sibling), SourceSettlementCurrentness::Dirty(_)),
                    dirty,
                    "the sibling's relation was added and removed again"
                );
            }
            let read_basis = Arc::new(read_basis);
            let readers = ["ClosureMemoReaderA", "ClosureMemoReaderB"].map(|name| {
                let reader = identity(world, coordinate, account, name);
                register(owner, Arc::clone(&reader), Arc::from([selection.clone()]), &read_basis, OrdSet::new());
                ConsumedOutputEvidence::retained_for_test(reader, Arc::from([selection.clone()]), vec![], None, Arc::clone(&read_basis))
            });
            let siblings = sibling_ids.map(|sibling| {
                ConsumedOutputEvidence::retained_for_test(sibling, Arc::clone(&sibling_facts), vec![], None, Arc::new(sibling_basis.clone()))
            });
            handle.with_runtime_for_test(execution, |runtime| runtime.snapshots().release_snapshot(&read_handle).unwrap());
            test(&Readers { world, owner, selection: selection.clone(), readers, siblings, account, coordinate });
        },
    );
}

fn resolve_observation(world: &AuthorizationWorld) -> (worth_runtime_world::facade::ProductBranchIncarnation, u64) {
    world
        .application
        .with_host_advancement(|phase| {
            let (_, product, _) = world.selected_product(&phase.test_host_request_for_test()).into_parts_for_test();
            let observation = product.observation_for_test();
            (observation.lifecycle_incarnation(), observation.reference_generation().get())
        })
        .expect("the fixture read owns its installed host call")
}

/// Run `body` at the latest native head, where the readers recorded their
/// selection and the siblings their dirt, with the selected product's
/// certification basis.
fn at_head<R>(
    world: &AuthorizationWorld,
    body: impl FnOnce(&worth_execution::ExecutionRequest<'_, '_>, &RelationalRuntime, CertificationBasis<'_>, &SnapshotHandle, &PositionedRelationalSnapshot) -> R,
) -> R {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let serial = worth_execution::ExecutionRequest::serial(&host_request);
    let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let selected = world.selected_product(&host);
            graph.integration_handle_for_test().with_runtime(&serial, |runtime| {
                let (snapshot, positioned) = snapshot(runtime);
                let product = selected.product();
                let basis =
                    CertificationBasis::of(product.lifecycle_incarnation(), product.reference_generation(), &snapshot);
                let result = body(&host.borrowed_request_for_test(), runtime, basis, &snapshot, &positioned);
                runtime.snapshots().release_snapshot(&snapshot).unwrap();
                result
            })
        })
        .expect("the fixture certification owns its installed host call")
}

/// Verify `[P2, S, P1]` in one walk. The marked walk pops `P1` first and
/// re-resolves its selection, then clears `S`, then reaches `P2`. Returns
/// the verdict and the settlement rows the walk compared.
fn walk(readers: &Readers<'_>) -> (String, u64) {
    at_head(readers.world, |execution, runtime, basis, _, positioned| {
        let roots = [readers.readers[1].clone(), readers.siblings[0].clone(), readers.readers[0].clone()];
        let before = cost().rows_compared;
        let verdict = ConsumedOutputEvidence::verify_many_with_admission_for_test(
            execution,
            &roots,
            readers.owner,
            runtime,
            basis,
            positioned,
        );
        (format!("{verdict:?}"), cost().rows_compared - before)
    })
}

/// Court 14 (h)(1): the walk clears `S` between the two readers' facts, so
/// `P2`'s re-resolution reads an image whose payload moved. Nothing it
/// compares changed meaning: the walk certifies `Current` with no retry,
/// and compares as many rows as the same walk with `S` already clean.
#[test]
fn a_cleared_sibling_between_two_selections_keeps_them_current() {
    readers(|readers| {
        let (cleared, cleared_rows) = walk(readers);
        assert_eq!(cleared, format!("{:?}", Ok::<_, ()>(ConsumedOutputVerification::Current)), "the walk certifies its readers");
        at_head(readers.world, |_, _, _, _, positioned| {
            assert!(
                matches!(currentness(readers.owner, positioned, readers.siblings[0].identity()), SourceSettlementCurrentness::Clean),
                "the walk cleared the dirty sibling"
            );
        });
        let (clean, clean_rows) = walk(readers);
        eprintln!("court 14 (h)(1): rows compared {cleared_rows} with the clearance, {clean_rows} without");
        assert_eq!(clean, cleared, "the same walk over a clean sibling certifies the same");
        assert_eq!(cleared_rows, clean_rows, "the clearance kept the closure memo");
    });
}

/// A certification frame the test opens, as one verification walk does:
/// certify the selection, run `between`, certify it again. Returns both
/// answers and the rows each compared.
fn certified_twice(
    readers: &Readers<'_>,
    between: impl FnOnce(&worth_execution::ExecutionRequest<'_, '_>, &RelationalRuntime, &SnapshotHandle, &PositionedRelationalSnapshot),
) -> [(String, u64); 2] {
    at_head(readers.world, |execution, runtime, basis, snapshot, positioned| {
        let _frame = CertificationScope::open(basis, positioned);
        let certify = || {
            let before = cost().rows_compared;
            let answer = match readers.selection.source_currentness_at(runtime, execution, basis) {
                Ok(Ok(movement)) if movement == FactMovement::from_equal(true) => "Unmoved".to_owned(),
                Ok(Ok(_)) => "Moved".to_owned(),
                Ok(Err(failure)) => format!("{failure:?}"),
                Err(stop) => format!("{stop:?}"),
            };
            (answer, cost().rows_compared - before)
        };
        let first = certify();
        between(execution, runtime, snapshot, positioned);
        [first, certify()]
    })
}

/// Court 14 (h), memo: inside one frame, a second certification of the same
/// selection takes the first one's closure. Clearing a dirty sibling between
/// them keeps that memo; an equal link recorded between them, or a
/// re-registration of the memoized head, clears it, and the second
/// certification compares the head's closure again.
#[test]
fn the_closure_memo_survives_a_clearance_and_not_a_changed_row() {
    readers(|readers| {
        let unchanged = certified_twice(readers, |_, _, _, _| {});
        eprintln!("court 14 (h) memo, nothing between: {unchanged:?}");
        assert_eq!(unchanged[0].0, "Unmoved", "the selection certifies");
        assert_eq!(unchanged[1].0, "Unmoved", "the selection certifies again");
        assert!(unchanged[1].1 < unchanged[0].1, "the second certification takes the memo: {unchanged:?}");

        let cleared = certified_twice(readers, |execution, runtime, snapshot, positioned| {
            let sibling = readers.siblings[1].identity();
            let DirtyReverification::Verified(verified) =
                readers.owner.reverify_dirty_for_test(execution, runtime, snapshot, positioned, sibling).unwrap()
            else {
                panic!("the sibling's restored relation set verifies at the head");
            };
            readers.owner.clear_verified_dirty_for_test(verified).unwrap();
        });
        eprintln!("court 14 (h) memo, a clearance between: {cleared:?}");
        assert_eq!(cleared, unchanged, "a clearance keeps the memo, and the second certification compares as few rows");

        let linked = certified_twice(readers, |execution, runtime, snapshot, positioned| {
            let graph = (*readers.world.application.runtime_for_test()).primary_graph_for_test().unwrap();
            let status_ref = AccountStatus::at("AccountStatus");
            let status = graph
                .layout_for_test()
                .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
                .unwrap()
                .clone();
            let predecessor = identity(readers.world, readers.coordinate, readers.account, "ClosureMemoLink");
            let successor = RecordedSettlementIdentity::retain_for_test(predecessor.source_for_test(), readers.coordinate, 1);
            register(
                readers.owner,
                Arc::clone(&predecessor),
                Arc::from([field_fact(execution, runtime, snapshot, readers.account, status.clone())]),
                positioned,
                OrdSet::new(),
            );
            register_alias(execution, readers.owner, runtime, snapshot, positioned, readers.account, status, &predecessor, &successor);
        });
        eprintln!("court 14 (h) memo, an equal link between: {linked:?}");
        assert_eq!(linked[0], unchanged[0], "the first certification is the same");
        assert_eq!(linked[1].0, "Unmoved", "the selection still certifies");
        assert_eq!(linked[1].1, linked[0].1, "a changed equal link clears the memo: the head is compared again");
    });
}

/// Court 14 (h), keep rule: re-registering the memoized head between two
/// certifications changes a row the memo read, so the memo is cleared and
/// the second certification compares the head again.
#[test]
fn a_reregistered_head_clears_the_closure_memo() {
    readers(|readers| {
        let (occurrence, generation) = resolve_observation(readers.world);
        let head = at_head(readers.world, |execution, _, _, _, _| {
            let graph = (*readers.world.application.runtime_for_test()).primary_graph_for_test().unwrap();
            let handle = graph.integration_handle_for_test();
            let lineage = handle.output_lineage.lock().unwrap();
            let mut resolution = lineage
                .resolve_current_family_for_test(
                    execution,
                    (*readers.world.application.runtime_for_test()).authority_identity_for_test().as_u64(),
                    &(*readers.world.application.installed_schema_for_test()).binding_identity(),
                    WorthQueryOperationScopeEntityBinding::from_entity_for_test(readers.account),
                    "test.retained-account-capacity",
                    occurrence,
                    generation,
                )
                .unwrap();
            let candidate = (*resolution.candidates_mut_for_test()).pop().expect("the family has its head");
            (Arc::clone(candidate.settlement_identity_for_test()), (*candidate.observed_source_facts_for_test()).clone())
        });
        let reregistered = certified_twice(readers, |_, _, _, positioned| {
            readers
                .owner
                .register_settlement_for_test(SettlementRegistration::new_for_test(
                    None,
                    None,
                    Arc::clone(&head.0),
                    head.1.clone(),
                    None,
                    positioned.clone(),
                    OrdSet::new(),
                    None,
                    OrdSet::new(),
                ))
                .expect("the head re-registers at the selected head");
        });
        eprintln!("court 14 (h) keep rule, a re-registered head between: {reregistered:?}");
        assert_eq!(reregistered[0].0, "Unmoved", "the selection certifies");
        assert!(
            reregistered[1].1 >= reregistered[0].1,
            "a re-registered memoized row clears the memo: the head is compared again: {reregistered:?}"
        );
    });
}
