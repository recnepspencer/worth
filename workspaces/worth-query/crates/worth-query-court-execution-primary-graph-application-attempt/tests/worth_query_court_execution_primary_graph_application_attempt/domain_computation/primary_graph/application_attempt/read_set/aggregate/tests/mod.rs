//! Exact issued-body custody and failed-call veto at actual admitted read sealing.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::context::court_doors_for_test::ContextDoorsOps;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationFactKey;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::denial;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::authorize;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionTraversalDenialKind;
use worth_execution::ExecutionAllocationPolicy as Allocation;
#[test]
fn aggregate_sealing_keeps_original_bodies_without_granting_relation_write_facts() {
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world.application.with_host_advancement(|phase| {
        let host=phase.test_host_request_for_test();
        let execution=host.execution_request_for_for_test(world.application.product_runtime()).unwrap();
        let (admission,root)=authorize(&world,&phase,&execution,&request,0,ReplaceNext::reference());
        let (_,projection,_)=world.invariant.project_admitted_operation(&phase,&admission,|reader,root| {
            reader.decision_orbit(Next::reference(),root).unwrap();
            reader.decision_orbit_of(Next::reference(),root).unwrap();
        },Allocation::SystemAllocation).unwrap().into_parts();
        let attempt=world.application.begin_projected_application_read_attempt(&phase,admission,projection,Allocation::SystemAllocation).unwrap();
        let expected=(*attempt.expected_facts_for_test()).as_ref().unwrap();
        let keys:Vec<_>=expected.iter_for_test().collect();
        assert_eq!(keys.len(),2);
        let WorthQueryApplicationFactKey::Aggregate(key)=*keys.iter().find(|key|matches!(key,WorthQueryApplicationFactKey::Aggregate(_))).unwrap() else {panic!("the aggregate and walk are distinct typed dependencies")};
        let original=std::sync::Arc::clone(expected.aggregate_for_test(key,(*attempt.admission_for_test()).admission_identity(),
            &(*attempt.admission_for_test()).retain_graph_contract(),(*attempt.lease_for_test()).snapshot_for_test()).unwrap());
        let (other,_)=authorize(&world,&phase,&execution,&request,0,ReplaceNext::reference());
        let denial=worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::aggregate::observe(&execution,&other,&(*attempt.lease_for_test()),&(*attempt.read_scope_for_test()),expected,key,None).err().unwrap();
        assert_eq!(denial.kind(),WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
            "even identical graph truth and contracts cannot relabel the actual value to another admission");
        let completed=attempt.complete_projected_dependencies(&phase,Allocation::SystemAllocation).unwrap();
        let values:Vec<_>=(*completed.facts_for_test()).iter_for_test().filter_map(|fact|match fact {
            Fact::Aggregate {observation}=>Some(observation),_=>None,
        }).collect();
        assert_eq!(values.len(),1);
        assert!(std::sync::Arc::ptr_eq(&original,values[0]),"seal consumed the exact original issued Arc");
        assert_eq!(original.native().len(),3);

        assert!(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::encoding::encode_funded(
            &[Fact::Aggregate { observation: original.clone() }],worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(Allocation::SystemAllocation,Some(&request))).is_err(),
            "live aggregate facts deliberately have no checkpoint reconstruction door");
        let selected=world.application.select_product_branch_for_test(&execution,world.application.product_runtime().default_branch_for_test()).unwrap();
        let target=selected.resolve_entity(&phase,NodeKey::reference(),1,&request,WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
        let denied=completed.projected_relation(Next::reference(),&root,&target).err().unwrap();
        assert_eq!(denied.kind(),WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
            "physical walk metadata does not manufacture the explicit pair fact");
        let effects=completed.begin_effect_program();
        assert!(effects.existing_entity(&target).is_ok(),"the funded visit route still recognizes an actually observed entity");
    }).unwrap();
}

#[test]
fn aggregate_descriptors_without_issued_values_cannot_complete() {
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let execution = host
                .execution_request_for_for_test(world.application.product_runtime())
                .unwrap();
            let (admission, _) = authorize(
                &world,
                &phase,
                &execution,
                &request,
                0,
                ReplaceNext::reference(),
            );
            let (_, recorded, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| {
                        reader.decision_orbit(Next::reference(), root).unwrap();
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let recorded = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    recorded,
                    Allocation::SystemAllocation,
                )
                .unwrap();
            let descriptor = (*recorded
                .expected_facts_for_test())
                .as_ref()
                .unwrap()
                .iter_for_test()
                .find_map(|key| match key {
                    WorthQueryApplicationFactKey::Aggregate(key) => Some(key.clone()),
                    _ => None,
                })
                .unwrap();
            let (admission, _) = authorize(
                &world,
                &phase,
                &execution,
                &request,
                0,
                ReplaceNext::reference(),
            );
            let (_, raw, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| {
                        reader.orbit(Next::reference(), root).unwrap();
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let raw = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    raw,
                    Allocation::SystemAllocation,
                )
                .unwrap();
            let denial = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::aggregate::observe(
                &execution,
                &(*raw.admission_for_test()),
                &(*raw.lease_for_test()),
                &(*raw.read_scope_for_test()),
                (*raw.expected_facts_for_test()).as_ref().unwrap(),
                &descriptor,
                None,
            )
            .err()
            .unwrap();
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch
            );
            let denied = (*raw
                .lease_for_test())
                .handle_for_test()
                .with_runtime(&execution, |runtime| {
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::fact_observation::observe_fact(
                        &execution,
                        runtime,
                        (*raw.lease_for_test()).snapshot_for_test(),
                        &(*(*raw.lease_for_test()).layout_for_test()),
                        &WorthQueryApplicationFactKey::Aggregate(descriptor),
                    )
                })
                .err()
                .unwrap();
            assert_eq!(
                denied.kind(),
                WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead
            );
        })
        .unwrap();
}

#[test]
fn failed_aggregate_calls_veto_sealing_before_a_key_exists_and_after_success() {
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let foreign = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let foreign_root = foreign
        .application
        .with_host_advancement(|phase| {
            foreign
                .invariant
                .project(
                    &phase,
                    |reader| reader.resolve_entity(NodeKey::reference(), 0).unwrap(),
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap();
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let execution = host
                .execution_request_for_for_test(world.application.product_runtime())
                .unwrap();
            for succeeded_first in [false, true] {
                let (admission, _) = authorize(
                    &world,
                    &phase,
                    &execution,
                    &request,
                    0,
                    ReplaceNext::reference(),
                );
                let (_, projection, _) = world
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, root| {
                            if succeeded_first {
                                reader.decision_orbit(Next::reference(), root).unwrap();
                            }
                            let error = reader
                                .decision_orbit(Next::reference(), &foreign_root)
                                .unwrap_err();
                            assert_eq!(
                                error.kind(),
                                WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity
                            );
                        },
                        Allocation::SystemAllocation,
                    )
                    .unwrap()
                    .into_parts();
                let denial = world
                    .application
                    .begin_projected_application_read_attempt(
                        &phase,
                        admission,
                        projection,
                        Allocation::SystemAllocation,
                    )
                    .err()
                    .unwrap();
                assert_eq!(
                    denial.kind(),
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    "the swallowed failed call has no successful or invented empty dependency"
                );
            }
        })
        .unwrap();
}
