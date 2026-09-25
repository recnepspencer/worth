use super::*;

#[test]
fn assessment_subject_selector_is_canonical_and_round_trips_persistence(
) -> Result<(), Box<dyn std::error::Error>> {
    let resource = ApplicationWorkflowSubjectSelector::Resource;
    let related = ApplicationWorkflowSubjectSelector::Related;
    let context = context_selector();
    assert_eq!(
        ApplicationWorkflowSubjectSelector::from_persistence_identity(
            &resource.persistence_identity(),
        ),
        Some(resource.clone()),
    );
    assert_eq!(
        ApplicationWorkflowSubjectSelector::from_persistence_identity(
            &related.persistence_identity(),
        ),
        Some(related.clone()),
    );
    let context_identity = context.persistence_identity();
    assert_eq!(
        ApplicationWorkflowSubjectSelector::from_persistence_identity(&context_identity),
        Some(context.clone()),
    );
    assert_eq!(
        ApplicationWorkflowSubjectSelector::from_persistence_identity(&format!(
            "{context_identity}trailing"
        )),
        None,
    );
    assert_eq!(
        ApplicationWorkflowSubjectSelector::from_persistence_identity(
            &context_identity[..context_identity.len() - 1],
        ),
        None,
    );
    let resource_definition = definition(resource)?;
    let related_definition = definition(related)?;
    assert_ne!(
        resource_definition.content_identity(),
        related_definition.content_identity(),
        "assessment subject coverage is canonical workflow meaning",
    );
    Ok(())
}

fn context_selector() -> ApplicationWorkflowSubjectSelector {
    ApplicationWorkflowSubjectSelector::context(
        crate::application_capability::ApplicationCapabilityContextEntitySlotBinding::from_untrusted_parts(
            crate::application_capability::WorthQueryPortableApplicationCapabilityContextEntitySlotBindingParts {
                context: "review|context:primary".to_owned(),
                context_identity: crate::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.query.tests|context:type".to_owned(),
                ),
                slot: "subject|slot:secondary".to_owned(),
                slot_identity: crate::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.query.tests|slot:type".to_owned(),
                ),
                entity: "part|entity:related".to_owned(),
            },
        ),
    )
}

fn definition(
    selector: ApplicationWorkflowSubjectSelector,
) -> Result<ValidatedWorkflowDefinition<ReviewedGeometry>, Box<dyn std::error::Error>> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "subject-selector",
        limits(),
    )?;
    let proposal = builder.operation::<ProposeChange>("proposal", false)?;
    let assessment = builder.assessment_for::<StructuralAssessment>("assessment", selector)?;
    let terminal = builder.terminal("completed")?;
    builder
        .start(&proposal)
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &assessment,
        )
        .control(
            &assessment,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        )
        .proposal_for_assessment(&proposal, &assessment);
    Ok(builder.finish()?.validate()?)
}
