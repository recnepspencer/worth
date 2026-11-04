//! Actor image and retention behavior, using real native source snapshots.
//! The existing test-only equality mint establishes actor coverage only.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::input_cutoff::stable_publication::source_registration::StableEqualityConsequence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;
use worth_relational::facade::mvcc::CompanionCellEditStop;

#[test]
fn repeated_current_certification_preserves_image_and_retention_and_rejects_a_racing_edit() {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request_for_test()))
        .expect("the fixture read owns its installed host call");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test()
        })
        .expect("the fixture read owns its installed host call");
    let (_, product, _) = selected.into_parts_for_test();
    let coordinate = ProductCoordinate {
        occurrence: product.observation_for_test().lifecycle_incarnation(),
        generation: product.observation_for_test().reference_generation().get(),
    };
    let source = SemanticSource {
        runtime_authority: (*world.application.runtime_for_test()).authority_identity().as_u64(),
        schema: (*world.application.installed_schema_for_test()).binding_identity(),
        scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity),
        output_binding: OutputBindingIdentity::declared_for_test("StatusOutput"),
    };
    let prior = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 0);
    let current = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 1);
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout_for_test()
        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout_for_test()
        .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let request_execution = &host_custody.borrowed_request_for_test();
            handle.with_runtime_mut_for_test(request_execution, |runtime| {
                write_field(request_execution, runtime, entity, label, "prime");
                let (snapshot, basis) = snapshot(runtime);
                let facts: Arc<[_]> = Arc::from([field_fact(
                    &host_custody.borrowed_request_for_test(),
                    runtime,
                    &snapshot,
                    entity,
                    status,
                )]);
                register(
                    owner,
                    Arc::clone(&prior),
                    Arc::clone(&facts),
                    &basis,
                    OrdSet::new(),
                );
                let relation = StableEqualityConsequence::native_actor_fixture_for_test(
                    Arc::clone(&prior),
                    Arc::clone(&current),
                    &basis,
                );
let alias = owner
            .prepare_current_stable_settlement_for_test(
                SettlementRegistration {
                    computation: None,
                    work_membership: None,
                    identity: Arc::clone(&current),
                    facts: RetainedSourceFacts::for_test_for_test(false, Arc::clone(&facts)),
                    output_facts: None,
                    read_basis: basis.clone(),
                    stale_at_read_basis: OrdSet::new(),
                    requirement: None,
                    upstream: OrdSet::new(),
                },
                &basis,
                relation,
            )
            .unwrap();


                drop(
                    alias
                        .install_for_test()
                        .unwrap_or_else(|_| panic!("selected alias installs")),
                );
let cell = owner
            .cell_for_read_for_test(&basis)
            .unwrap()
            .unwrap();


                for with_downstream in [false, true] {
                    if with_downstream {
                        let downstream = RecordedSettlementIdentity::retain_for_test(
                            &SemanticSource {
                                output_binding: OutputBindingIdentity::declared_for_test("DownstreamOutput"),
                                ..source.clone()
                            },
                            coordinate,
                            0,
                        );
                        register(
                            owner,
                            Arc::clone(&downstream),
                            Arc::from([]),
                            &basis,
                            OrdSet::unit(Arc::clone(&current)),
                        );
                        assert!(matches!(
                            currentness(owner, &basis, &downstream),
                            SourceSettlementCurrentness::Clean
                        ));
                    }
                    let original = cell.read_image();

                    for _ in 0..3 {

let prepared = owner
                    .prepare_verified_current_for_test(
                        runtime,
                        &snapshot,
                        &basis,
                        &current,
                        &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(false, Arc::clone(&facts)).for_comparison().unwrap(),
                    )
                    .unwrap()
                    .expect("complete clean actor coverage certifies current");
drop(
                            prepared
                                .install_for_test()
                                .unwrap_or_else(|_| panic!("unchanged image certifies")),
                        );
                        let observed = cell.read_image();
                        assert!(Arc::ptr_eq(original.payload(), observed.payload()));
                        assert_eq!(
                            original.topology_generation(),
                            observed.topology_generation()
                        );

                    }
                }

                let original = cell.read_image();


let stale = owner
            .prepare_verified_current_for_test(
                runtime,
                &snapshot,
                &basis,
                &current,
                &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(false, Arc::clone(&facts)).for_comparison().unwrap(),
            )
            .unwrap()
            .unwrap();
let other = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 2);
                register(
                    owner,
                    Arc::clone(&other),
                    Arc::clone(&facts),
                    &basis,
                    OrdSet::new(),
                );
                let replacement = cell.read_image();
                assert!(!Arc::ptr_eq(original.payload(), replacement.payload()));
                let stopped = match stale.install_for_test() {
                    Err(stopped) => stopped,
                    Ok(_) => panic!("an actor edit must reject the old image certificate"),
                };
                assert_eq!(
                    stopped.reason_for_test(),
                    CompanionCellEditStop::TopologyGenerationChanged
                );
                drop(stopped);
                assert!(Arc::ptr_eq(
                    replacement.payload(),
                    cell.read_image().payload()
                ));
                assert!(matches!(
                    currentness(owner, &basis, &other),
                    SourceSettlementCurrentness::Clean
                ));
                runtime.snapshots().release_snapshot(&snapshot).unwrap();
            })
        })
        .expect("the fixture read owns its installed host call");
}
