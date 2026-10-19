//! Complete native birth producer/read regression, not a public mutation-authorization journey.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        workflow_instance_observation as observation, WorthQueryApplicationCreationPartition,
    },
    workflow::instance::visit_instance_birth_facts,
};
use std::collections::BTreeMap;
use worth_query_declaration::facade::{
    application_program::ApplicationWorkflowComponentLimits,
    application_schema::ApplicationEntityMarkerIdentity,
};
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;
use worth_relational::facade::symbols::ClientKey;

#[test]
fn birth_subject_provenance_and_start_actor_remain_readable() {
    let world = installed_authorization_world(true);
    let (support, revision, _) = installed_provenance_support(world.application.installed_schema());
    let payer = payer();
    let execution = ExecutionRequest::leased(&payer);
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph.layout.workflow().clone();
    let (_, product, _lease) = world
        .application
        .select_product_branch(
            &execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap()
        .into_parts();
    let branch = product.product_branch();
    let actor = admitted_actor(&world);
    let compiled = super::definition_fixture::publish_definition(
        &world,
        &execution,
        &layout,
        revision,
        "birth-read",
        actor.entity().partition_id,
    );
    let subject_kind = graph.layout.entity_kind(Account::IDENTIFIER).unwrap();
    let before = selected(&world, &execution);
    let inventories = graph
        .integration_handle()
        .with_runtime(&execution, |runtime| {
            [subject_kind, layout.instance.entity_kind].map(|kind| {
                runtime
                    .read_truth()
                    .visible_entities_of_kind(
                        &execution,
                        kind,
                        before.1.snapshot_handle().version_id(),
                    )
                    .unwrap()
                    .into_iter()
                    .map(|record| record.entity_id)
                    .collect::<Vec<_>>()
            })
        });
    let created = CreatedEntityRef {
        partition_id: compiled.definition().partition_id,
        kind_id: subject_kind,
        client_key: ClientKey::raw("birth-subject"),
    };
    let value = graph
        .integration_handle()
        .with_runtime(&execution, |runtime| {
            start_provenance::admit(&execution, runtime, &before.0, &revision).unwrap()
        });
    let mut effects = vec![Effect::CreateEntity {
        kind: subject_kind,
        key: "birth-subject".to_owned(),
        fields: BTreeMap::new(),
        partition: WorthQueryApplicationCreationPartition::Context(created.partition_id),
    }];
    visit_instance_birth_facts(
        &layout,
        &compiled,
        "birth-read",
        &value,
        branch.occurrence_ordinal(),
        created,
        WorkflowLineageCarry::default(),
        actor,
        |effect| {
            effects.push(effect);
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    publish_effects(
        &world,
        &execution,
        compiled.definition().partition_id,
        effects,
    );
    let subject = only_new_entity(&world, &execution, subject_kind, &inventories[0]);
    let instance = only_new_entity(
        &world,
        &execution,
        layout.instance.entity_kind,
        &inventories[1],
    );
    let current = selected(&world, &execution);
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        8,
        8,
        32,
        ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1).unwrap(),
        4096,
        8,
        32,
        4096,
    )
    .unwrap();
    graph
        .integration_handle()
        .with_runtime(&execution, |runtime| {
            let record = observation::governed_record::observe(
                &execution,
                runtime,
                current.1.snapshot_handle(),
                &layout,
                branch,
                instance,
                true,
                compiled.lineage(),
                &revision,
                &support,
                resources,
            )
            .expect("governed reader accepts complete native birth facts");
            assert_eq!(record.subject, subject);
            assert_eq!(record.original_start_revision, revision);
            assert!(
                record.reference.is_some(),
                "current birth retains an actual command locator"
            );
            let mut facts = Vec::new();
            observation::observe_subject(
                &execution,
                runtime,
                current.1.snapshot_handle(),
                &layout,
                instance,
                subject,
                &mut facts,
            )
            .expect("recorded-actor scope consumes the same birth subject");
            assert!(
                observation::observe_subject(
                    &execution,
                    runtime,
                    current.1.snapshot_handle(),
                    &layout,
                    instance,
                    actor.entity(),
                    &mut Vec::new(),
                )
                .is_err(),
                "another real subject cannot substitute for the birth endpoint"
            );
            let start = layout
                .instance
                .start_advancer
                .observe(
                    runtime,
                    &execution,
                    current.1.snapshot_handle(),
                    instance,
                    layout.instance.entity_kind,
                    &mut Vec::new(),
                )
                .unwrap();
            assert_eq!(start, Some(actor.entity()));
        });
    // Deliberate native corruption, not an alternate valid encoding: the
    // authoritative birth edge must never coexist with coordinate claims.
    publish_relational_mutation(
        &execution,
        &world,
        update(
            instance,
            &layout.instance.subject_slot,
            AspectValue::UInt64(subject.local_slot_value()),
        ),
    );
    let corrupted = selected(&world, &execution);
    graph
        .integration_handle()
        .with_runtime(&execution, |runtime| {
            assert!(
                observation::observe_subject(
                    &execution,
                    runtime,
                    corrupted.1.snapshot_handle(),
                    &layout,
                    instance,
                    subject,
                    &mut Vec::new(),
                )
                .is_err(),
                "mixed edge and coordinate subject authority is rejected"
            );
        });
}
