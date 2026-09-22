use super::*;

const LARGE_CANONICAL_BYTE_LIMIT: u32 = 16 * 1024 * 1024;
const TEN_THOUSAND_NODE_INDEX_BYTE_ENVELOPE: u64 = 4 * 1024 * 1024;
const RETRY_RICH_CANONICAL_BYTE_LIMIT: u32 = 1024 * 1024;
const RETRY_RICH_INDEX_BYTE_ENVELOPE: u64 = 256 * 1024;

#[test]
fn ten_thousand_node_sparse_definition_has_indexed_validation_work(
) -> Result<(), Box<dyn std::error::Error>> {
    const NODES: u16 = 10_000;
    let limits = ApplicationWorkflowDefinitionLimits::new(
        NODES,
        NODES,
        NODES - 1,
        ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1).unwrap(),
        LARGE_CANONICAL_BYTE_LIMIT,
    )
    .unwrap();
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "ten-thousand-node-chain",
        limits,
    )?;
    let first = builder.operation::<ProposeChange>("node-00000", false)?;
    builder.start(&first);
    let mut previous = first;
    for index in 1..usize::from(NODES - 1) {
        let next = builder.operation::<ProposeChange>(format!("node-{index:05}"), false)?;
        builder.control(
            &previous,
            ApplicationWorkflowControlOutcome::Completed,
            &next,
        );
        previous = next;
    }
    let terminal = builder.terminal("node-09999")?;
    builder.control(
        &previous,
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );

    let validated = builder.finish()?.validate()?;
    let work = validated.validation_work();
    assert_eq!(
        work.complexity_contract(),
        ApplicationWorkflowValidationComplexityContract::IndexedSparseGraphWithLogarithmicDominance
    );
    assert_eq!(work.limit_nodes(), 10_000);
    assert_eq!(work.provenance_records(), 0);
    assert_eq!(work.indexed_nodes(), 10_000);
    assert_eq!(work.indexed_connections(), 9_999);
    assert_eq!(work.connection_semantics(), 9_999);
    assert_eq!(work.identity_lookups(), 19_999);
    assert_eq!(work.requirement_nodes(), 10_000);
    assert_eq!(work.control_nodes(), 30_000);
    assert_eq!(work.control_edges(), 19_998);
    assert_eq!(work.dominance_candidates(), 9_999);
    assert_eq!(work.dominance_predecessors(), 0);
    assert_eq!(work.dominance_table_writes(), 0);
    assert_eq!(work.dominance_queries(), 0);
    assert_eq!(work.dominance_lifts(), 0);
    assert_eq!(work.retry_reachability(), 0);
    assert_eq!(work.total_visits(), 129_994);
    assert!(work.peak_index_bytes() > 0);
    assert!(work.peak_index_bytes() <= TEN_THOUSAND_NODE_INDEX_BYTE_ENVELOPE);
    Ok(())
}

#[test]
fn ten_thousand_node_data_chain_exercises_logarithmic_dominance_work(
) -> Result<(), Box<dyn std::error::Error>> {
    const NODES: u16 = 10_000;
    let limits = ApplicationWorkflowDefinitionLimits::new(
        NODES,
        NODES * 2,
        NODES - 1,
        ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1).unwrap(),
        LARGE_CANONICAL_BYTE_LIMIT,
    )
    .unwrap();
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "ten-thousand-node-data-chain",
        limits,
    )?;
    let first = builder.operation::<ProposeChange>("node-00000", false)?;
    builder.start(&first);
    let mut previous = first;
    for index in 1..usize::from(NODES - 1) {
        let next = builder.operation::<ProposeChange>(format!("node-{index:05}"), false)?;
        builder
            .control(
                &previous,
                ApplicationWorkflowControlOutcome::Completed,
                &next,
            )
            .operation_input(&previous, &next);
        previous = next;
    }
    let terminal = builder.terminal("node-09999")?;
    builder.control(
        &previous,
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );

    let work = builder.finish()?.validate()?.validation_work();
    assert_eq!(work.indexed_nodes(), 10_000);
    assert_eq!(work.indexed_connections(), 19_997);
    assert_eq!(work.dominance_candidates(), 2);
    assert_eq!(work.dominance_predecessors(), 9_999);
    assert_eq!(work.dominance_table_writes(), 129_987);
    assert_eq!(work.dominance_queries(), 9_998);
    assert_eq!(work.dominance_lifts(), 9_998);
    assert_eq!(work.total_visits(), 319_971);
    assert!(work.peak_index_bytes() <= TEN_THOUSAND_NODE_INDEX_BYTE_ENVELOPE);
    Ok(())
}

#[test]
fn retry_rich_validation_reports_its_separate_sparse_traversal_lane(
) -> Result<(), Box<dyn std::error::Error>> {
    const OPERATIONS: u16 = 100;
    let limits = ApplicationWorkflowDefinitionLimits::new(
        OPERATIONS + 1,
        OPERATIONS * 2,
        OPERATIONS,
        ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1).unwrap(),
        RETRY_RICH_CANONICAL_BYTE_LIMIT,
    )
    .unwrap();
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "retry-rich-validation",
        limits,
    )?;
    let first = builder.operation::<ProposeChange>("operation-000", false)?;
    builder.start(&first);
    let mut operations = vec![first];
    for index in 1..usize::from(OPERATIONS) {
        operations
            .push(builder.operation::<ProposeChange>(format!("operation-{index:03}"), false)?);
    }
    let terminal = builder.terminal("terminal")?;
    for (index, operation) in operations.iter().enumerate() {
        builder.retry(
            operation,
            ApplicationWorkflowRetry::new(
                ApplicationWorkflowControlOutcome::Completed,
                "bounded-revision",
                1,
            )
            .unwrap(),
            &operations[0],
        );
        if let Some(exhausted) = operations.get(index + 1) {
            builder.control(
                operation,
                ApplicationWorkflowControlOutcome::RetryExhausted,
                exhausted,
            );
        } else {
            builder.control(
                operation,
                ApplicationWorkflowControlOutcome::RetryExhausted,
                &terminal,
            );
        }
    }

    let work = builder.finish()?.validate()?.validation_work();
    assert_eq!(
        work.retry_complexity_contract(),
        ApplicationWorkflowRetryValidationComplexityContract::PerRetrySparseControlTraversal
    );
    assert_eq!(work.retry_reachability(), 20_100);
    assert!(work.peak_index_bytes() <= RETRY_RICH_INDEX_BYTE_ENVELOPE);
    Ok(())
}
