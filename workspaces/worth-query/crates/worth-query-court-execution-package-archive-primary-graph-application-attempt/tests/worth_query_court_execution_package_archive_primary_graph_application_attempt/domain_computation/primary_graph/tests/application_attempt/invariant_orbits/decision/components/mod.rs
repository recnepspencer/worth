//! Original admitted roots, whole Native cohorts, sealing and physical invalidation.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::Empty;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::Focused;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitSchema;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::Selection;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::Shells;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::read_selection;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::currentness;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::component_selection::ComponentSelection;
use worth_execution::ExecutionRequest;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionTraversalDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::component_selection;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::denial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::root;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::*;
use std::collections::BTreeSet;
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy as Allocation;
#[test]
fn admitted_component_of_and_complete_census_seal_original_native_bodies() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world.application.with_host_advancement(|phase| {
        let host=phase.test_host_request();let execution=host.execution_request_for(world.application.product_runtime()).unwrap();
        let (admission,_)=worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::authorize(&world,&phase,&execution,&request,0,CheckComponents::reference());
        let (_,snapshot,_)=world.invariant.project_admitted_operation(&phase,&admission,|reader,root| {
            let component=reader.component_of(Shells::reference(),root).unwrap();
            let keys=component.members().map(|member|reader.field(&member,NodeKey::reference()).unwrap()).collect::<BTreeSet<_>>();
            assert_eq!(keys,BTreeSet::from([0,1,2]));
            let census=reader.components(Shells::reference()).unwrap();
            assert_eq!(census.len(),1);assert_eq!(census[0].identity(),component.identity());
            assert_eq!(reader.component_count(Shells::reference()).unwrap(),1);
            assert_eq!(reader.component_count(Empty::reference()).unwrap(),0);
        },Allocation::SystemAllocation).unwrap().into_parts();
        let sealed=world.application.begin_projected_application_read_attempt(&phase,admission,snapshot,Allocation::SystemAllocation).unwrap().complete_projected_dependencies(&phase,Allocation::SystemAllocation).unwrap();
        let facts=sealed.decision_facts();
        let bodies=facts.iter().filter_map(|fact|match fact {WorthQueryApplicationObservedFact::ComponentSelection {observation}=>Some(observation),_=>None}).collect::<Vec<_>>();
        assert_eq!(bodies.len(),3,"Of, complete Census, and genuine empty Census remain distinct; repeated count reuses its exact key");
        assert!(bodies.iter().any(|body|body.aggregates().is_empty()));
        // Exercise the actual ordinary handler entrance as well as the lower
        // admitted projection. Its candidate retains the same complete reads.
        let (admission,_)=worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::authorize(&world,&phase,&execution,&request,0,CheckComponents::reference());
        let input=worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::CheckComponentsInput {root:0};
        let key="component-handler".to_owned();
        let identities=worth_query_declaration::facade::application_operation::ApplicationMutationIdentities::<OrbitSchema<true>,CheckComponentsBinding<OrbitSchema<true>>>::encode(&key,&input).unwrap();
        let report=world.application.execute_mutation_handler_report::<CheckComponentsBinding<OrbitSchema<true>>>(&phase,&identities,&1_u64,admission,Allocation::SystemAllocation);
        let worth_query_execution::facade::domain_computation::HandlerResult::Completed(completed)=report.into_outcome().unwrap() else {panic!("real component handler must complete");};
        let (_candidate,result)=completed.into_parts();
        assert_eq!(result,worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::binding::CheckOrbitOutcome(3));

    }).unwrap();
}
#[test]
fn partial_or_foreign_cohorts_refuse_and_swallowed_failures_cannot_seal() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    for foreign in [false, true] {
        world.application.with_host_advancement(|phase| {
            let host=phase.test_host_request();let execution=host.execution_request_for(world.application.product_runtime()).unwrap();
            let (admission,_)=worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::authorize(&world,&phase,&execution,&request,0,CheckComponents::reference());
            let (_,snapshot,_)=world.invariant.project_admitted_operation(&phase,&admission,|reader,root| {
                let error=if foreign {let other=reader.resolve_entity(NodeKey::reference(),3_u64).unwrap();reader.component_of(Shells::reference(),&other).unwrap_err()} else {reader.component_of(Focused::reference(),root).unwrap_err()};
                assert_eq!(error.kind(),WorthQueryInvariantProjectionTraversalDenialKind::ComponentOutsideScope);
            },Allocation::SystemAllocation).unwrap().into_parts();
            let denial=world.application.begin_projected_application_read_attempt(&phase,admission,snapshot,Allocation::SystemAllocation).err().unwrap();
            assert_eq!(denial.kind(),WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact);
        }).unwrap();
    }
}
#[test]
fn empty_census_tracks_negative_guard_facts_and_unrelated_edits_remain_current() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let original = read_selection(&world, 0, Selection::Empty, &request);
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    assert!(original
        .lease
        .handle()
        .with_runtime(&execution, |runtime| original
            .body
            .remains_equal(runtime, &execution, original.lease.snapshot(), None)
            .unwrap()));
    let (other, key) = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::currentness::prepare(&world, &request, 6, "components-unrelated");
    assert!(matches!(
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::currentness::publish(&world, &request, other, key),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let fresh = read_selection(&world, 0, Selection::Empty, &request);
    assert!(fresh
        .lease
        .handle()
        .with_runtime(&execution, |runtime| original
            .body
            .remains_equal(runtime, &execution, fresh.lease.snapshot(), None)
            .unwrap()));
    let (changed, key) = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::currentness::prepare(&world, &request, 0, "components-guard-change");
    assert!(matches!(
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::currentness::publish(&world, &request, changed, key),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    // Acquire the new root without repeating the now-nonempty census: a partial
    // visible cohort must refuse, while the old complete empty answer is stale.
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let selected = world
                .application
                .select_product_branch(
                    &execution,
                    world.application.product_runtime().default_branch(),
                )
                .unwrap();
            let projection = world
                .invariant
                .project(&phase, |_| (), Allocation::SystemAllocation)
                .unwrap();
            let (_, snapshot, _) = projection.into_parts();
            let (lease, _, _, _) =
                snapshot.into_lease_and_realized_scope(selected.product().retained_clone());
            assert!(!lease.handle().with_runtime(&execution, |runtime| original
                .body
                .remains_equal(runtime, &execution, lease.snapshot(), None)
                .unwrap()));
        })
        .unwrap();
}
#[test]
fn coupling_revision_stales_a_selection_when_visibility_membership_and_native_name_match() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let originals = [
        read_selection(&world, 0, Selection::Shells, &request),
        read_selection(&world, 3, Selection::Shells, &request),
    ];
    let winner = usize::from(
        originals[0].body.aggregates()[0].native().anchor()
            > originals[1].body.aggregates()[0].native().anchor(),
    );
    let original = &originals[winner];
    for key in [241, 242] {
        let edit = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::swap_successors(&world, &request, 0, 3);
        assert!(matches!(
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::commit(&world, &request, edit, key),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }
    let fresh = read_selection(
        &world,
        if winner == 0 { 0 } else { 3 },
        Selection::Shells,
        &request,
    );
    let before = &original.body.aggregates()[0];
    let after = &fresh.body.aggregates()[0];
    assert_eq!(
        before.native().members().collect::<Vec<_>>(),
        after.native().members().collect::<Vec<_>>()
    );
    assert_eq!(before.native().identity(), after.native().identity());
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    fresh.lease.handle().with_runtime(&execution,|runtime| {
        assert!(before.remains_equal(runtime,&execution,fresh.lease.snapshot(),None).unwrap(),"the Native aggregate is independently unchanged");
        assert!(!original.body.remains_equal(runtime,&execution,fresh.lease.snapshot(),None).unwrap(),"only physical coupling revisions changed; the independent ComponentVisibility relation was never edited");
    });
}
