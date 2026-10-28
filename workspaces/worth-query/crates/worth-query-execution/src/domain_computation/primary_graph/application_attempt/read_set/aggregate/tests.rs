//! Exact issued-body custody and failed-call veto at actual admitted read sealing.
use super::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::{
    decision::identity, program::authorize, schema::*,
};
use crate::domain_computation::primary_graph::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;
#[test]
fn aggregate_sealing_keeps_original_bodies_without_granting_relation_write_facts() {
    let world = identity::install();
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    world.application.with_host_advancement(|phase| {
        let host=phase.test_host_request();
        let execution=host.execution_request_for(world.application.product_runtime()).unwrap();
        let (admission,root)=authorize(&world,&phase,&execution,&request,0,ReplaceNext::reference());
        let (_,projection,_)=world.invariant.project_admitted_operation(&phase,&admission,|reader,root| {
            reader.decision_orbit(Next::reference(),root).unwrap();
            reader.decision_orbit_of(Next::reference(),root).unwrap();
        },Allocation::SystemAllocation).unwrap().into_parts();
        let attempt=world.application.begin_projected_application_read_attempt(&phase,admission,projection,Allocation::SystemAllocation).unwrap();
        let expected=attempt.expected_facts.as_ref().unwrap();
        let keys:Vec<_>=expected.iter().collect();
        assert_eq!(keys.len(),2);
        let WorthQueryApplicationFactKey::Aggregate(key)=*keys.iter().find(|key|matches!(key,WorthQueryApplicationFactKey::Aggregate(_))).unwrap() else {panic!("the aggregate and walk are distinct typed dependencies")};
        let original=std::sync::Arc::clone(expected.aggregate(key,attempt.admission.admission_identity(),
            &attempt.admission.retain_graph_contract(),attempt.lease.snapshot()).unwrap());
        let (other,_)=authorize(&world,&phase,&execution,&request,0,ReplaceNext::reference());
        let denial=super::observe(&execution,&other,&attempt.lease,&attempt.read_scope,expected,key,None).err().unwrap();
        assert_eq!(denial.kind(),WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
            "even identical graph truth and contracts cannot relabel the actual value to another admission");
        let completed=attempt.complete_projected_dependencies(&phase,Allocation::SystemAllocation).unwrap();
        let values:Vec<_>=completed.facts.iter().filter_map(|fact|match fact {
            Fact::Aggregate {observation}=>Some(observation),_=>None,
        }).collect();
        assert_eq!(values.len(),1);
        assert!(std::sync::Arc::ptr_eq(&original,values[0]),"seal consumed the exact original issued Arc");
        assert_eq!(original.native().len(),3);

        assert!(crate::domain_computation::primary_graph::application_checkpoint::facts::encode_funded(
            &[Fact::Aggregate { observation: original.clone() }],super::super::super::retained_decision_facts::StorageControl::new(Allocation::SystemAllocation,Some(&request))).is_err(),
            "live aggregate facts deliberately have no checkpoint reconstruction door");
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
fn aggregate_descriptors_without_issued_values_cannot_complete() {
    let world = identity::install();
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
            let descriptor = recorded
                .expected_facts
                .as_ref()
                .unwrap()
                .iter()
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
    let world = identity::install();
    let foreign = identity::install();
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
