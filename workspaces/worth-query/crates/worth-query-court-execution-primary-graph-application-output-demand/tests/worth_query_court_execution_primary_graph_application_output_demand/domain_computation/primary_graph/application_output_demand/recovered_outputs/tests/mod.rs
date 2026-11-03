use std::sync::Arc;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::recovered_outputs::WorthQueryRecoveredOutputs;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointIdentity;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryReadmittedAcceptedOutput;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;

pub fn partition(number: u64) -> [u8; 32] {
    let mut identity = [0; 32];
    identity[..8].copy_from_slice(&number.to_le_bytes());
    identity
}

pub fn unrelated_restored_selection(population: u64) {
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity(
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
        );
    let correspondence = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
    let host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut metadata = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::metadata_funding::CheckpointMetadataFunding::new(&execution).unwrap();
    metadata
        .slots::<WorthQueryReadmittedAcceptedOutput>((population + 1) as usize, &execution)
        .unwrap();
    metadata
        .bytes("producer".len() * (population + 1) as usize, &execution)
        .unwrap();
    metadata
        .slots::<String>((population + 1) as usize, &execution)
        .unwrap();
    metadata.slots::<Vec<worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryCheckpointOutputRole>>((population + 1) as usize, &execution).unwrap();
    let records = (0..=population)
            .map(|number| WorthQueryReadmittedAcceptedOutput {
                checkpoint: WorthQueryAcceptedOutputCheckpointIdentity {
                    implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation::from_checkpoint([0x11; 16], [0x22; 32]).unwrap(),
                    producer: ("producer".to_owned()).into(),
                    posture: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::accepted_checkpoint_identity::WorthQueryAcceptedOutputCheckpointPosture::Performed,
                    source: [0x31; 32],
                    scope,
                    source_partition: partition(number),
                    producer_dependency: None,
                    idempotency_key: [0x41; 32],
                    roles: (Vec::new()).into(),
                    producer_facts: None,
                    producer_fact_wire_version: 0,
                    metadata: None,
                },
                correspondence: Arc::clone(&correspondence),
            })
            .collect();
    let recovered =
        WorthQueryRecoveredOutputs::from_records(records, metadata, &execution).unwrap();
    assert_eq!(recovered.iter().count(), (population + 1) as usize);
    let selected = recovered
        .matching_source_partition(scope, partition(0))
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].checkpoint.source_partition, partition(0));
    assert!(recovered
        .matching_source_partition(scope, [0xff; 32])
        .next()
        .is_none());
}

#[test]
fn restored_selection_excludes_1k_and_10k_unrelated_partitions() {
    for population in [1_000, 10_000] {
        unrelated_restored_selection(population);
    }
}

#[test]
fn restored_selection_excludes_100k_unrelated_partitions() {
    unrelated_restored_selection(100_000);
}
