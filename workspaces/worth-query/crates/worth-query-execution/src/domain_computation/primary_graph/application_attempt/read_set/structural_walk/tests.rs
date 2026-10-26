//! Exact issued-body custody and failed-call veto at actual admitted read sealing.
use super::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::{
    program::authorization::authorize, schema::*, world,
};
use crate::domain_computation::primary_graph::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_query_declaration::facade::application_schema::RelationDirection::Forward;
#[test]
fn sealing_keeps_the_one_original_issued_body_and_requires_explicit_relation_facts() {
    let world = world::install(world::Graph::three_hops(false, vec![]));
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    world.application.with_host_advancement(|phase| {
        let host=phase.test_host_request();
        let execution=host.execution_request_for(world.application.product_runtime()).unwrap();
        let (admission,root)=authorize(&world,&phase,&execution,&request,0,InspectWalk::reference(),None);
        let (_,projection,_)=world.invariant.project_admitted_operation(&phase,&admission,|reader,root| {
            reader.decision_walk(Next::reference(),Forward,root).unwrap();
            reader.decision_closure(Next::reference(),Forward,&[root.clone(),root.clone()]).unwrap();
            assert!(reader.decision_reaches(Next::reference(),root,root).unwrap());
        },Allocation::SystemAllocation).unwrap().into_parts();
        let attempt=world.application.begin_projected_application_read_attempt(&phase,admission,projection,Allocation::SystemAllocation).unwrap();
        let expected=attempt.expected_facts.as_ref().unwrap();
        let keys:Vec<_>=expected.iter().collect();
        assert_eq!(keys.len(),1);
        let WorthQueryApplicationFactKey::StructuralWalk(key)=keys[0] else {panic!("the complete walk is one typed dependency")};
        let original=std::sync::Arc::clone(expected.structural_walk(key,attempt.admission.admission_identity(),
            &attempt.admission.retain_graph_contract(),attempt.lease.snapshot()).unwrap());
        let (other,_)=authorize(&world,&phase,&execution,&request,0,InspectWalk::reference(),None);
        let denial=super::observe(&execution,&other,&attempt.lease,&attempt.read_scope,expected,key,None).err().unwrap();
        assert_eq!(denial.kind(),WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
            "even identical graph truth and contracts cannot relabel the actual value to another admission");
        let completed=attempt.complete_projected_dependencies(&phase,Allocation::SystemAllocation).unwrap();
        let values:Vec<_>=completed.facts.iter().filter_map(|fact|match fact {
            Fact::StructuralWalk {observation}=>Some(observation),_=>None,
        }).collect();
        assert_eq!(values.len(),1);
        assert!(std::sync::Arc::ptr_eq(&original,values[0]),"seal consumed the exact original issued Arc");
        assert_eq!(original.native().visits().len(),4);
        assert_eq!(original.native().relations().len(),3);
        assert!(crate::domain_computation::primary_graph::application_checkpoint::facts::encode_funded(
            &completed.facts,super::super::super::retained_decision_facts::StorageControl::new(Allocation::SystemAllocation,Some(&request))).is_err(),
            "live walk facts deliberately have no checkpoint reconstruction door");
        let selected=world.application.select_product_branch(&execution,world.application.product_runtime().default_branch()).unwrap();
        let target=selected.resolve_entity(&phase,NodeKey::reference(),1,&request,WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
        let denied=completed.projected_relation(Next::reference(),&root,&target).err().unwrap();
        assert_eq!(denied.kind(),WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
            "physical walk metadata does not manufacture the explicit pair fact");
        let effects=completed.begin_effect_program();
        assert!(effects.existing_entity(&target).is_ok(),"the funded visit route still recognizes an actually observed entity");
    }).unwrap();
}

#[test]
fn a_descriptor_without_this_projections_issued_value_cannot_be_completed() {
    let world = world::install(world::Graph::three_hops(false, vec![]));
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            let (admission, _) = authorize(
                &world,
                &phase,
                &execution,
                &request,
                0,
                InspectWalk::reference(),
                None,
            );
            let (_, recorded, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| {
                        reader
                            .decision_walk(Next::reference(), Forward, root)
                            .unwrap();
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
            let descriptor = recorded
                .expected_facts
                .as_ref()
                .unwrap()
                .iter()
                .find_map(|key| match key {
                    WorthQueryApplicationFactKey::StructuralWalk(key) => Some(key.clone()),
                    _ => None,
                })
                .unwrap();
            let (admission, _) = authorize(
                &world,
                &phase,
                &execution,
                &request,
                0,
                InspectWalk::reference(),
                None,
            );
            let (_, raw, _) = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, root| {
                        reader.walk(Next::reference(), Forward, root).unwrap();
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
            let denial = super::observe(
                &execution,
                &raw.admission,
                &raw.lease,
                &raw.read_scope,
                raw.expected_facts.as_ref().unwrap(),
                &descriptor,
                None,
            )
            .err()
            .unwrap();
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch
            );
            let denied = raw
                .lease
                .handle()
                .with_runtime(&execution, |runtime| {
                    super::super::fact_observation::observe_fact(
                        &execution,
                        runtime,
                        raw.lease.snapshot(),
                        &raw.lease.layout,
                        &WorthQueryApplicationFactKey::StructuralWalk(descriptor),
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
fn failed_calls_veto_sealing_before_a_key_exists_and_after_a_successful_duplicate() {
    let world = world::install(world::Graph::three_hops(false, vec![]));
    let foreign = world::install(world::Graph::three_hops(false, vec![]));
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
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            for succeeded_first in [false, true] {
                let (admission, _) = authorize(
                    &world,
                    &phase,
                    &execution,
                    &request,
                    0,
                    InspectWalk::reference(),
                    None,
                );
                let (_, projection, _) = world
                    .invariant
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, root| {
                            if succeeded_first {
                                reader
                                    .decision_walk(Next::reference(), Forward, root)
                                    .unwrap();
                            }
                            let error = reader
                                .decision_walk(Next::reference(), Forward, &foreign_root)
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
