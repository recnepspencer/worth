use super::*;

#[test]
fn ordinary_sequence_matches_explicit_control_without_inventing_data(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut ordinary =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("solver-chain", limits())?;
    let structure = ordinary.operation::<ProposeChange>("structure", false)?;
    let electrical = ordinary.operation::<ApplyChange>("electrical", false)?;
    let done = ordinary.terminal("done")?;
    ordinary.start(&structure);
    ordinary.sequence(&[structure.clone(), electrical.clone()])?;
    ordinary.control(
        &electrical,
        ApplicationWorkflowControlOutcome::Completed,
        &done,
    );
    let ordinary = ordinary.finish()?.validate()?;

    let mut explicit =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("solver-chain", limits())?;
    let structure = explicit.operation::<ProposeChange>("structure", false)?;
    let electrical = explicit.operation::<ApplyChange>("electrical", false)?;
    let done = explicit.terminal("done")?;
    explicit
        .start(&structure)
        .control(
            &structure,
            ApplicationWorkflowControlOutcome::Completed,
            &electrical,
        )
        .control(
            &electrical,
            ApplicationWorkflowControlOutcome::Completed,
            &done,
        );
    let explicit = explicit.finish()?.validate()?;
    assert_eq!(ordinary.content_identity(), explicit.content_identity());
    assert_eq!(ordinary.connections(), explicit.connections());
    assert_eq!(ordinary.connections().len(), 2);
    Ok(())
}

#[test]
fn ordinary_sequence_rejects_foreign_nodes_without_partial_connections(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("sequence", limits())?;
    let local = builder.operation::<ProposeChange>("local", false)?;
    let mut foreign =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("other", limits())?;
    let absent = foreign.operation::<ApplyChange>("absent", false)?;
    let mistaken = foreign.operation::<ApplyChange>("collision", false)?;
    builder.assessment::<StructuralAssessment>("collision")?;
    assert!(matches!(
        builder.sequence(&[local.clone(), absent]),
        Err(ApplicationWorkflowAuthoringDenial::UnknownSequenceNode(_))
    ));
    assert!(matches!(
        builder.sequence(&[local.clone(), mistaken]),
        Err(ApplicationWorkflowAuthoringDenial::NonOperationSequenceNode(_))
    ));
    assert!(matches!(
        builder.sequence(&[local.clone(), local.clone()]),
        Err(ApplicationWorkflowAuthoringDenial::DuplicateSequenceNode(_))
    ));
    assert!(matches!(
        builder.sequence(&[local]),
        Err(ApplicationWorkflowAuthoringDenial::InvalidSequenceLength)
    ));
    assert!(builder.finish()?.connections.is_empty());
    Ok(())
}

#[test]
fn ordinary_sequence_remains_bounded_at_ten_thousand_nodes(
) -> Result<(), Box<dyn std::error::Error>> {
    const NODES: u16 = 10_000;
    let limits = ApplicationWorkflowDefinitionLimits::new(
        NODES,
        NODES,
        NODES - 1,
        ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1).unwrap(),
        16 * 1024 * 1024,
    )
    .unwrap();
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new("large-sequence", limits)?;
    let mut operations = Vec::with_capacity(usize::from(NODES - 1));
    for index in 0..usize::from(NODES - 1) {
        operations
            .push(builder.operation::<ProposeChange>(format!("operation-{index:05}"), false)?);
    }
    let terminal = builder.terminal("done")?;
    builder.start(&operations[0]);
    builder.sequence(&operations)?;
    builder.control(
        operations.last().unwrap(),
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );
    let validated = builder.finish()?.validate()?;
    assert_eq!(validated.nodes().len(), usize::from(NODES));
    assert_eq!(validated.connections().len(), usize::from(NODES - 1));
    Ok(())
}
