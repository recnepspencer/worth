use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::resolution::latest_output_matching;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_source_identity::RecordedSourceIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::semantic_source::SemanticSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;

pub struct RestoredOutputBinding;

#[test]
fn restoration_keeps_sibling_parameter_partitions_in_one_generation_slot() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let product = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .expect("the fixture's default product occurrence is live");
    let observation = product.observation_for_test();
    let runtime_authority = (*world.application.runtime_for_test())
        .authority_identity_for_test()
        .as_u64();
    let schema = (*world.application.installed_schema_for_test()).binding_identity();
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(
        worth_relational::facade::identity::EntityId::new(
            worth_relational::facade::identity::PartitionId::main(),
            1,
            1,
        ),
    );
    let first_partition = [0x11; 32];
    let sibling_partition = [0x22; 32];
    let first = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let sibling = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        "RestoredOutputBinding",
    );

    lineage.record_restoration_for_test(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::clone(&first),
        checkpoint_identity([0x31; 32]),
        first_partition,
        None,
        [0x41; 32],
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(source_facts()),
        None,
        None,
        None,
    );
    lineage.record_restoration_for_test(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::clone(&sibling),
        checkpoint_identity([0x32; 32]),
        sibling_partition,
        Some([0x52; 32]),
        [0x42; 32],
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(source_facts()),
        None,
        None,
        None,
    );
    lineage.record_restoration_for_test(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        runtime_authority,
        schema.clone(),
        scope,
        observation,
        Arc::clone(&first),
        checkpoint_identity([0x31; 32]),
        first_partition,
        None,
        [0x41; 32],
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(source_facts()),
        None,
        None,
        None,
    );

    let source = SemanticSource::new_for_test(
        runtime_authority,
        schema,
        scope,
        OutputBindingIdentity::declared_for_test("RestoredOutputBinding"),
    );
    let history = &(*lineage.by_source_for_test())[&source][&observation.lifecycle_incarnation()];
    let maximum_generation = observation.reference_generation().get();
    let coordinate =
        ProductCoordinate::new_for_test(observation.lifecycle_incarnation(), maximum_generation);
    let selected_first = lineage
        .latest_output_in_partition_for_test(&source, coordinate, first_partition)
        .unwrap();
    let selected_sibling = lineage
        .latest_output_in_partition_for_test(&source, coordinate, sibling_partition)
        .unwrap();
    let matching_sibling = latest_output_matching(history, maximum_generation, |recorded| {
        (*recorded.source_partition_identity_for_test()) == Some(sibling_partition)
    })
    .unwrap();

    assert!(Arc::ptr_eq(
        &(*selected_first.correspondence_for_test()),
        &first
    ));
    assert!(Arc::ptr_eq(
        &(*selected_sibling.correspondence_for_test()),
        &sibling
    ));
    assert!(Arc::ptr_eq(
        &(*matching_sibling.correspondence_for_test()),
        &sibling
    ));
    assert_eq!(history[&maximum_generation].len(), 2);

    lineage.install_output_families_for_test(std::collections::BTreeMap::from([(
        "partitioned-family".to_owned().into(),
        vec![(
            TypeId::of_for_test::<RestoredOutputBinding>(),
            "output".to_owned(),
        )],
    )]));
    let resolve = || {
        lineage.resolve_current_family_for_test(
            execution,
            runtime_authority,
            &(*source.schema_for_test()),
            scope,
            "partitioned-family",
            observation.lifecycle_incarnation(),
            maximum_generation,
        )
    };
    let family = resolve().expect("binding, partition heads, and bounded publication selection");

    assert_eq!((*family.candidates_for_test()).len(), 2);
    assert!(Arc::ptr_eq(
        &(*(*family.candidates_for_test())[0].correspondence_for_test()),
        &first
    ));
    assert!(Arc::ptr_eq(
        &(*(*family.candidates_for_test())[1].correspondence_for_test()),
        &sibling
    ));

    let current_checkpoint =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity::new_for_test([0x31; 32]);
    let current_runtime =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity::new_for_test([0x99; 32]);
    let candidates = lineage
        .retained_output_candidates_metered_for_test(
            runtime_authority,
            &(*source.schema_for_test()),
            scope,
            observation.lifecycle_incarnation(),
            maximum_generation,
            &[TypeId::of_for_test::<RestoredOutputBinding>()],
            first_partition,
            &worth_execution::ExecutionRequest::serial(
                &worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request(),
            ),
        )
        .unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        (*candidates[0].source_identity_for_test()),
        Some(checkpoint_identity([0x31; 32]))
    );
    assert_eq!(
        (*lineage
            .qualified_output_for_test::<RestoredOutputBinding>(
                runtime_authority,
                &(*source.schema_for_test()),
                scope,
                observation.lifecycle_incarnation(),
                maximum_generation,
                current_runtime,
                current_checkpoint,
            )
            .expect("the restored record matches its portable source identity")
            .source_identity_for_test()),
        checkpoint_identity([0x31; 32]),
    );
}

pub fn checkpoint_identity(identity: [u8; 32]) -> RecordedSourceIdentity {
    RecordedSourceIdentity::Checkpoint(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity::new_for_test(identity),
    )
}

pub fn source_facts(
) -> Arc<[worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact]>{
    Arc::from([
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact::SourceEntity {
            entity_id: worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
        },
    ])
}

#[test]
fn family_inventory_visits_follow_declared_names() {
    use std::collections::BTreeMap;
    for _ in 0..16 {
        for reverse in [false, true] {
            let mut declarations = [
                ("zulu-family".to_owned().into(), Vec::new()),
                ("alpha-family".to_owned().into(), Vec::new()),
            ];
            if reverse {
                declarations.reverse();
            }
            let mut lineage = WorthQueryApplicationOutputLineage::default();
            lineage.install_output_families_for_test(BTreeMap::from(declarations));
            let visits: Vec<_> = (*lineage.output_families_for_test())
                .keys()
                .map(|family| family.as_str_for_test())
                .collect();
            assert_eq!(visits, ["alpha-family", "zulu-family"]);
        }
    }
}

#[path = "family_publications/mod.rs"]
pub mod family_publications;
#[path = "input_cutoff_selection/mod.rs"]
pub mod input_cutoff_selection;
#[path = "partition_selection/mod.rs"]
pub mod partition_selection;
#[path = "prior_correspondence/mod.rs"]
pub mod prior_correspondence;
#[path = "republication/mod.rs"]
pub mod republication;
#[path = "restoration_identity/mod.rs"]
pub mod restoration_identity;
