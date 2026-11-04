//! Complete native birth producer/read regression, not a public mutation-authorization journey.
use crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction::start_provenance_tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationCreationPartition;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::instance::facts::visit_instance_birth_facts;
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_program::ApplicationWorkflowComponentLimits;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;
use worth_relational::facade::symbols::ClientKey;

#[test]
fn birth_subject_provenance_and_start_actor_remain_readable() {
    let world = installed_authorization_world(true);
    let (support, revision, _) = installed_provenance_support(world.application.installed_schema());
    let payer = payer();
    let execution = ExecutionRequest::leased(&payer);
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
    let layout = (*graph.layout_for_test()).workflow().clone();
    let (_, product, _lease) = world
        .application
        .select_product_branch_for_test(
            &execution,
            world.application.product_runtime().default_branch_for_test(),
        )
        .unwrap()
        .into_parts_for_test();
    let branch = product.product_branch();
    let actor = admitted_actor(&world);
    let compiled = crate::domain_computation::primary_graph::workflow::definition::compilation::reconstruction::start_provenance_tests::definition_fixture::publish_definition(
        &world,
        &execution,
        &layout,
        revision,
        "birth-read",
        actor.entity_for_test().partition_id,
    );
    let subject_kind = (*graph.layout_for_test()).entity_kind_for_test(Account::IDENTIFIER).unwrap();
    let before = selected(&world, &execution);
    let inventories = graph
        .integration_handle_for_test()
        .with_runtime(&execution, |runtime| {
            [subject_kind, (*(*layout.instance_for_test()).entity_kind_for_test())].map(|kind| {
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
        partition_id: compiled.definition_for_test().partition_id,
        kind_id: subject_kind,
        client_key: ClientKey::raw("birth-subject"),
    };
    let value = graph
        .integration_handle_for_test()
        .with_runtime(&execution, |runtime| {
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::workflow::instance::start_provenance::admit(&execution, runtime, &before.0, &revision).unwrap()
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
        compiled.definition_for_test().partition_id,
        effects,
    );
    let subject = only_new_entity(&world, &execution, subject_kind, &inventories[0]);
    let instance = only_new_entity(
        &world,
        &execution,
        (*(*layout.instance_for_test()).entity_kind_for_test()),
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
        .integration_handle_for_test()
        .with_runtime(&execution, |runtime| {
            let record = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_instance_observation::governed_record::observe(
                &execution,
                runtime,
                current.1.snapshot_handle(),
                &layout,
                branch,
                instance,
                true,
                compiled.lineage_for_test(),
                &revision,
                &support,
                resources,
            )
            .expect("governed reader accepts complete native birth facts");
            assert_eq!((*record.subject_for_test()), subject);
            assert_eq!((*record.original_start_revision_for_test()), revision);
            assert!(
                (*record.reference_for_test()).is_some(),
                "current birth retains an actual command locator"
            );
            let mut facts = Vec::new();
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_instance_observation::subject::observe_subject(
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
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_instance_observation::subject::observe_subject(
                    &execution,
                    runtime,
                    current.1.snapshot_handle(),
                    &layout,
                    instance,
                    actor.entity_for_test(),
                    &mut Vec::new(),
                )
                .is_err(),
                "another real subject cannot substitute for the birth endpoint"
            );
            let start = (*(*layout
                .instance_for_test())
                .start_advancer_for_test())
                .observe_for_test(
                    runtime,
                    &execution,
                    current.1.snapshot_handle(),
                    instance,
                    (*(*layout.instance_for_test()).entity_kind_for_test()),
                    &mut Vec::new(),
                )
                .unwrap();
            assert_eq!(start, Some(actor.entity_for_test()));
        });
    // Deliberate native corruption, not an alternate valid encoding: the
    // authoritative birth edge must never coexist with coordinate claims.
    publish_relational_mutation(
        &execution,
        &world,
        update(
            instance,
            &(*(*layout.instance_for_test()).subject_slot_for_test()),
            AspectValue::UInt64(subject.local_slot_value()),
        ),
    );
    let corrupted = selected(&world, &execution);
    graph
        .integration_handle_for_test()
        .with_runtime(&execution, |runtime| {
            assert!(
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_instance_observation::subject::observe_subject(
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
