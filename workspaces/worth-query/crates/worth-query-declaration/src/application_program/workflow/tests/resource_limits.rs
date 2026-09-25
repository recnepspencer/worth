use super::*;

#[test]
fn component_limit_axes_are_distinct_and_independently_capped() {
    let ceiling = ApplicationWorkflowComponentLimits::new(2, 3, 5, 7, 11).unwrap();
    assert_eq!(ceiling.maximum_occurrences(), 2);
    assert_eq!(ceiling.maximum_depth(), 3);
    assert_eq!(ceiling.maximum_node_provenance(), 5);
    assert_eq!(ceiling.maximum_connection_provenance(), 7);
    assert_eq!(ceiling.maximum_port_provenance(), 11);
    assert!(ceiling.fits_within(ceiling));

    for exceeded in [
        ApplicationWorkflowComponentLimits::new(3, 3, 5, 7, 11).unwrap(),
        ApplicationWorkflowComponentLimits::new(2, 4, 5, 7, 11).unwrap(),
        ApplicationWorkflowComponentLimits::new(2, 3, 6, 7, 11).unwrap(),
        ApplicationWorkflowComponentLimits::new(2, 3, 5, 8, 11).unwrap(),
        ApplicationWorkflowComponentLimits::new(2, 3, 5, 7, 12).unwrap(),
    ] {
        assert!(!exceeded.fits_within(ceiling));
    }
}

#[test]
fn node_and_connection_provenance_have_independent_denials(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut component =
        ApplicationWorkflowComponentBuilder::<ReviewedGeometry>::new("bounded-graph")?;
    let first = component.assessment::<StructuralAssessment>("first")?;
    let second = component.assessment::<StructuralAssessment>("second")?;
    let third = component.assessment::<StructuralAssessment>("third")?;
    component.control(
        &first,
        ApplicationWorkflowControlOutcome::Completed,
        &second,
    )?;
    component.control(
        &second,
        ApplicationWorkflowControlOutcome::Completed,
        &third,
    )?;
    let component = component.finish()?;

    for (limits, expected) in [
        (
            ApplicationWorkflowComponentLimits::new(1, 1, 2, 2, 3).unwrap(),
            ApplicationWorkflowComponentResource::NodeProvenance,
        ),
        (
            ApplicationWorkflowComponentLimits::new(1, 1, 3, 1, 3).unwrap(),
            ApplicationWorkflowComponentResource::ConnectionProvenance,
        ),
    ] {
        let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
            "independent-provenance",
            ApplicationWorkflowDefinitionLimits::new(3, 2, 1, limits, 2_048).unwrap(),
        )?;
        assert!(matches!(
            definition.expand_component("graph", &component),
            Err(
                ApplicationWorkflowAuthoringDenial::ComponentResourceLimitExceeded {
                    resource,
                    ..
                }
            ) if resource == expected
        ));
        assert!(definition.finish()?.component_expansions.is_empty());
    }
    Ok(())
}

#[test]
fn component_provenance_limits_deny_before_mutating_the_definition(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut component =
        ApplicationWorkflowComponentBuilder::<ReviewedGeometry>::new("bounded-ports")?;
    let internal = component.assessment::<StructuralAssessment>("internal")?;
    component.input_port("first", &internal)?;
    component.input_port("second", &internal)?;
    let component = component.finish()?;

    let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "bounded-component-provenance",
        ApplicationWorkflowDefinitionLimits::new(
            1,
            1,
            1,
            ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1).unwrap(),
            1_024,
        )
        .unwrap(),
    )?;
    assert!(matches!(
        definition.expand_component("review", &component),
        Err(
            ApplicationWorkflowAuthoringDenial::ComponentResourceLimitExceeded {
                resource: ApplicationWorkflowComponentResource::PortProvenance,
                maximum: 1,
            }
        )
    ));
    let authored = definition.finish()?;
    assert!(authored.nodes.is_empty());
    assert!(authored.connections.is_empty());
    assert!(authored.component_expansions.is_empty());
    Ok(())
}

#[test]
fn empty_components_consume_independent_occurrence_and_depth_limits(
) -> Result<(), Box<dyn std::error::Error>> {
    let inner =
        ApplicationWorkflowComponentBuilder::<ReviewedGeometry>::new("empty-inner")?.finish()?;
    let mut outer = ApplicationWorkflowComponentBuilder::<ReviewedGeometry>::new("empty-outer")?;
    outer.expand_component("nested", &inner)?;
    let outer = outer.finish()?;

    let component_limits = ApplicationWorkflowComponentLimits::new(2, 1, 8, 8, 8).unwrap();
    let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "empty-component-bounds",
        ApplicationWorkflowDefinitionLimits::new(8, 8, 1, component_limits, 4_096).unwrap(),
    )?;
    assert!(matches!(
        definition.expand_component("outer", &outer),
        Err(
            ApplicationWorkflowAuthoringDenial::ComponentResourceLimitExceeded {
                resource: ApplicationWorkflowComponentResource::ComponentDepth,
                maximum: 1,
            }
        )
    ));
    definition.expand_component("first", &inner)?;
    definition.expand_component("second", &inner)?;
    assert!(matches!(
        definition.expand_component("third", &inner),
        Err(
            ApplicationWorkflowAuthoringDenial::ComponentResourceLimitExceeded {
                resource: ApplicationWorkflowComponentResource::ComponentOccurrences,
                maximum: 2,
            }
        )
    ));
    assert_eq!(definition.finish()?.component_expansions.len(), 2);
    Ok(())
}
