//! A Native leaf touch queues its ancestor scopes and no sibling subtree.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::required_work::RequiredWorkWorthQueryOutputDemandRegistryForTest;
use worth_query_execution::facade::domain_computation::WorthQueryNativeMarkingPrecision;
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::SelectedRequiredWorkKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::CreateIntent;
use worth_relational::facade::transactions::CreatedEntityRef;
use worth_relational::facade::transactions::EntitySpec;
pub const DEPTH: usize = 6;

#[test]
fn a_deep_native_leaf_has_identical_work_with_eight_or_sixty_four_sibling_subtrees() {
    {
        let mut counts = None;
        for siblings in [8, 64] {
            let observed = run(siblings);
            assert_eq!(
                *counts.get_or_insert(observed),
                observed,
                "the leaf's logical work depends on its ancestor path, not the sibling population"
            );
        }
    }
}

pub fn run(
    siblings: usize,
) -> worth_query_execution::facade::domain_computation::WorthQueryNativeMarkingWork {
    let world = crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::population_world::install((siblings + 1) * DEPTH);
    world.application.with_host_advancement(|phase| {
 let custody = phase.test_host_request_for_test();
 let phase_custody = &custody;
 let execution = &phase_custody.borrowed_request_for_test();

    let (_, product, _) = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .into_parts_for_test();
    let coordinate = ProductCoordinate::new_for_test(product.observation_for_test().lifecycle_incarnation(), product.observation_for_test().reference_generation().get());
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let kind = graph
        .layout_for_test()
        .entity_kind_for_test(Account::reference().name())
        .unwrap();
    let locator = |entity, aspect, field| {
        graph
            .layout_for_test()
            .field_locator_for_test(entity, aspect, field)
            .unwrap()
            .clone()
    };
    let i = AccountIdentity::reference();
    let s = AccountStatus::reference();
    let l = AccountLabel::reference();
    let identity = locator(i.entity(), i.aspect(), i.field());
    let status = locator(s.entity(), s.aspect(), s.field());
    let label = locator(l.entity(), l.aspect(), l.field());
    handle.with_runtime_mut_for_test(execution, |runtime| {
        let refs: Vec<_> = (0..(siblings + 1) * DEPTH).map(|n| CreatedEntityRef {
            partition_id: PartitionId::main(), kind_id: kind,
            client_key: ClientKey::raw(format!("scope-tree-{n}")),
        }).collect();
        let batch = refs.iter().enumerate().fold(WorkerIntentBatch::new("scope-forest"), |batch, (n, r)| {
            let value = |text| AspectValue::String(InternedString::Raw(text));
            batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: r.partition_id, kind_id: r.kind_id, client_key: r.client_key.clone(),
                fields: AspectFieldPatch::from(BTreeMap::from([
                    (identity.clone(), value(format!("scope-tree-{n}"))),
                    (status.clone(), value("neutral".to_owned())),
                    (label.clone(), value("before".to_owned())),
                ])),
            })))
        });
        let committed = write_batch(execution, runtime, batch);
        let entities: Vec<_> = refs.iter().map(|r| committed.created_entity(r).unwrap()).collect();
        release_test_commit_snapshot(runtime, &committed);
        let (before_handle, before) = snapshot(runtime);
        let registries: Vec<_> = entities.iter().map(|_| WorthQueryOutputDemandRegistry::default()).collect();
        let members: Vec<_> = registries.iter().zip(&entities).map(|(registry, entity)|
            registry.fixture_scope_work_membership((*coordinate.occurrence_for_test()), *entity)).collect();
        let outputs: Vec<_> = entities.iter().map(|entity| RecordedSettlementIdentity::retain_for_test(&SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(*entity), OutputBindingIdentity::declared_for_test("ScopeOutput")), coordinate, 0)).collect();
        for n in 0..entities.len() {
            // Each distinct Native scope above the leaf consumes exactly its
            // child's output. Sibling paths have no consumed edge in common.
            let leaf = n % DEPTH == 0;
            owner.register_settlement_for_test(SettlementRegistration::new_for_test(None, Some(Arc::clone(&members[n].1)), Arc::clone(&outputs[n]), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(false,
                    if leaf { Arc::from([field_fact(execution, runtime, &before_handle, entities[n], label.clone())]) } else { Arc::from([]) }), None, before.clone(), OrdSet::new(), None, if leaf { OrdSet::new() } else { OrdSet::unit(Arc::clone(&outputs[n - 1])) })).unwrap();
            // Admission queues initial disclosure; registration queues one
            // local settlement cue. Both precede the measured Native edit.
            for _ in 0..2 {


let initial = registries[n].next_required_work_for_test(owner).unwrap().unwrap();
assert!(matches!(initial.kind_for_test(), SelectedRequiredWorkKind::Local { .. } | SelectedRequiredWorkKind::UnresolvedInitial));
                initial.acknowledge_fixture();
            }
            assert!(registries[n].next_required_work_for_test(owner).unwrap().is_none());
        }
        write_field(execution, runtime, entities[0], label, "after");
        let (after_handle, after) = snapshot(runtime);
let report = owner.native_marking_report_for_test(&after).unwrap().unwrap();


        let WorthQueryNativeMarkingPrecision::Exact(counts) = (*report.precision_for_test()) else { panic!("the declared Native leaf touch is exact") };
        assert_eq!(counts.marked_fact_ordinals, 1);
        assert_eq!(counts.visited_vertices, DEPTH as u64);
        assert_eq!(counts.downstream_edges, DEPTH as u64 - 1);
        for (n, registry) in registries.iter().enumerate() {

            let candidate = registry.next_required_work_for_test(owner).unwrap();
            if n < DEPTH {
                let candidate = candidate.expect("each ancestor has one Native pending cue");
                assert!(matches!(candidate.kind_for_test(), SelectedRequiredWorkKind::Native { .. }));
                candidate.acknowledge_fixture();
            } else {
                assert!(candidate.is_none(), "sibling scope {n} contributes zero candidates, hence zero Ready selections");
                assert!(matches!(currentness(owner, &after, &outputs[n]), SourceSettlementCurrentness::Clean));
            }
            assert!(registry.next_required_work_for_test(owner).unwrap().is_none(), "one cue per dirty scope");
        }
        for snapshot in [before_handle, after_handle] { runtime.snapshots().release_snapshot(&snapshot).unwrap(); }
        counts
    })

 }).expect("the native journey owns its installed host call")
}
