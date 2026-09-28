use super::*;

#[test]
fn primitive_component_macro_and_command_authoring_are_canonical_equivalents(
) -> Result<(), Box<dyn std::error::Error>> {
    let primitive = primitive_definition()?;
    let component = component_definition()?;
    let mut macro_component =
        ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("required-change-review")?;
    let macro_consistency = macro_component.assessment::<ConsistencyAssessment>("consistency")?;
    let macro_compliance = macro_component.assessment::<ComplianceAssessment>("compliance")?;
    let macro_evidence = macro_component.evidence_join(
        "evidence",
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )?;
    macro_component.control(
        &macro_consistency,
        ApplicationWorkflowControlOutcome::Completed,
        &macro_compliance,
    )?;
    macro_component.control(
        &macro_compliance,
        ApplicationWorkflowControlOutcome::Completed,
        &macro_evidence,
    )?;
    macro_component.assessment_evidence(&macro_consistency, &macro_evidence)?;
    macro_component.assessment_evidence(&macro_compliance, &macro_evidence)?;
    let macro_consistency_input =
        macro_component.input_port("consistency-subject", &macro_consistency)?;
    let macro_compliance_input =
        macro_component.input_port("compliance-subject", &macro_compliance)?;
    let macro_evidence_output =
        macro_component.output_port("reviewed-evidence", &macro_evidence)?;
    let macro_component = macro_component.finish()?;
    let macro_authored = crate::worth_query_workflow! {
        spec: ReviewedChange;
        identity: "reviewed-change";
        limits: limits();
        build: |builder| {
            let propose = builder.operation::<ProposeChange>("propose", false)?;
            let approval = builder.approval::<ChangeApprover>("approval")?;
            let apply = builder.operation::<ApplyChange>("apply", true)?;
            let completed = builder.terminal("completed")?;
            let rejected = builder.terminal("rejected")?;
            let checks = builder.expand_component("checks", &macro_component)?;
            let consistency = checks.input(&macro_consistency_input)?;
            let compliance = checks.input(&macro_compliance_input)?;
            let evidence = checks.output(&macro_evidence_output)?;
            builder
                .start(&propose)
                .control(&propose, ApplicationWorkflowControlOutcome::Completed, &consistency)
                .control(&evidence, ApplicationWorkflowControlOutcome::EvidenceSatisfied, &approval)
                .control(&evidence, ApplicationWorkflowControlOutcome::EvidenceFailed, &rejected)
                .control(&approval, ApplicationWorkflowControlOutcome::Approved, &apply)
                .control(&approval, ApplicationWorkflowControlOutcome::Rejected, &rejected)
                .control(&apply, ApplicationWorkflowControlOutcome::Completed, &completed)
                .proposal_for_assessment(&propose, &consistency)
                .proposal_for_assessment(&propose, &compliance)
                .proposal_for_approval(&propose, &approval)
                .joined_evidence(&evidence, &approval)
                .approval_authority(&approval, &apply)
                .operation_input(&propose, &apply);
        }
    }?
    .validate()?;
    let command = command_definition()?;
    assert_eq!(primitive.content_identity(), component.content_identity());
    assert_eq!(
        primitive.content_identity(),
        macro_authored.content_identity()
    );
    assert_eq!(primitive.content_identity(), command.content_identity());
    assert_eq!(primitive.nodes(), component.nodes());
    assert_eq!(primitive.nodes(), macro_authored.nodes());
    assert_eq!(primitive.nodes(), command.nodes());
    assert_eq!(primitive.connections(), component.connections());
    let [expansion] = component.component_expansions() else {
        panic!("the component occurrence must retain one expansion record")
    };
    assert_eq!(expansion.component().as_str(), "required-change-review");
    assert_eq!(expansion.occurrence_path(), "checks");
    assert_eq!(expansion.nodes().len(), 3);
    assert_eq!(expansion.ports().len(), 3);
    assert_eq!(expansion.connections().len(), 4);
    assert_eq!(expansion.ports()[0].identity(), "consistency-subject");
    assert_eq!(
        expansion.ports()[0].direction(),
        ApplicationWorkflowComponentPortDirection::Input
    );
    assert_eq!(
        expansion.ports()[0].expanded_node().as_str(),
        "checks/consistency"
    );
    assert_eq!(expansion.nodes()[0].authored().as_str(), "consistency");
    assert_eq!(
        expansion.nodes()[0].expanded().as_str(),
        "checks/consistency"
    );
    Ok(())
}

#[test]
fn component_occurrences_are_qualified_and_collision_denial_is_atomic(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut component =
        ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("reusable-review")?;
    let internal = component.assessment::<ConsistencyAssessment>("internal")?;
    component.input_port("subject", &internal)?;
    let component = component.finish()?;

    let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "qualified-components",
        limits(),
    )?;
    definition.expand_component("first", &component)?;
    definition.expand_component("second", &component)?;
    assert!(matches!(
        definition.expand_component("first", &component),
        Err(ApplicationWorkflowAuthoringDenial::DuplicateComponentOccurrence(occurrence))
            if occurrence == "first"
    ));
    let authored = definition.finish()?;
    assert_eq!(authored.component_expansions.len(), 2);
    assert_eq!(authored.nodes[0].identity().as_str(), "first/internal");
    assert_eq!(authored.nodes[1].identity().as_str(), "second/internal");

    let mut collision_component =
        ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("collision-review")?;
    collision_component.assessment::<ConsistencyAssessment>("first")?;
    collision_component.assessment::<ConsistencyAssessment>("second")?;
    let collision_component = collision_component.finish()?;
    let mut collision = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "qualified-collision",
        limits(),
    )?;
    collision.operation::<ProposeChange>("review/second", false)?;
    assert!(matches!(
        collision.expand_component("review", &collision_component),
        Err(ApplicationWorkflowAuthoringDenial::DuplicateNode(identity))
            if identity.as_str() == "review/second"
    ));
    let collision = collision.finish()?;
    assert_eq!(collision.nodes.len(), 1);
    assert!(collision.connections.is_empty());
    assert!(collision.component_expansions.is_empty());

    let mut reverse = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "reverse-qualified-collision",
        limits(),
    )?;
    reverse.expand_component("review", &collision_component)?;
    assert!(matches!(
        reverse.operation::<ProposeChange>("review/first", false),
        Err(ApplicationWorkflowAuthoringDenial::DuplicateNode(identity))
            if identity.as_str() == "review/first"
    ));
    let reverse = reverse.finish()?;
    assert_eq!(reverse.nodes.len(), 2);
    assert_eq!(reverse.component_expansions.len(), 1);
    Ok(())
}

#[test]
fn nested_component_ports_and_authored_provenance_retain_complete_occurrence_paths(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut inner = ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("inner-review")?;
    let assessment = inner.assessment::<ConsistencyAssessment>("assessment")?;
    let inner_input = inner.input_port("subject", &assessment)?;
    let inner_output = inner.output_port("evidence", &assessment)?;
    let inner = inner.finish()?;

    let mut outer = ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("outer-review")?;
    let inner_occurrence = outer.expand_component("nested", &inner)?;
    let nested_input = inner_occurrence.input(&inner_input)?;
    let nested_output = inner_occurrence.output(&inner_output)?;
    outer.input_port("subject", &nested_input)?;
    outer.output_port("evidence", &nested_output)?;
    let outer = outer.finish()?;

    let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "nested-component-provenance",
        limits(),
    )?;
    definition.expand_component("review", &outer)?;
    let authored = definition.finish()?;
    let [outer_expansion, inner_expansion] = authored.component_expansions.as_slice() else {
        panic!("both outer and nested component occurrences must retain provenance")
    };
    assert_eq!(outer_expansion.occurrence_path(), "review");
    assert_eq!(inner_expansion.occurrence_path(), "review/nested");
    assert_eq!(
        inner_expansion.ports()[0].expanded_node().as_str(),
        "review/nested/assessment"
    );
    Ok(())
}

#[test]
fn component_ports_are_occurrence_bound_and_internal_nodes_are_not_public_bindings(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut first = ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("first-review")?;
    let first_node = first.assessment::<ConsistencyAssessment>("internal")?;
    let first_input = first.input_port("subject", &first_node)?;
    let first = first.finish()?;

    let mut second = ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("second-review")?;
    let second_node = second.assessment::<ConsistencyAssessment>("internal")?;
    let second_input = second.input_port("subject", &second_node)?;
    let second = second.finish()?;

    let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(
        "component-port-affinity",
        limits(),
    )?;
    let occurrence = definition.expand_component("review", &first)?;
    assert_eq!(
        occurrence.input(&first_input)?.identity().as_str(),
        "review/internal"
    );
    assert!(matches!(
        occurrence.input(&second_input),
        Err(ApplicationWorkflowAuthoringDenial::ForeignComponentPort)
    ));

    let mut impostor = ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("first-review")?;
    let impostor_node = impostor.assessment::<ConsistencyAssessment>("internal")?;
    let impostor_input = impostor.input_port("subject", &impostor_node)?;
    let _ = impostor.finish()?;
    assert!(matches!(
        occurrence.input(&impostor_input),
        Err(ApplicationWorkflowAuthoringDenial::ForeignComponentPort)
    ));
    let _ = second;
    Ok(())
}

#[test]
fn same_identity_component_handles_cannot_cross_builder_ownership(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut first = ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("first-owner")?;
    let foreign = first.assessment::<ConsistencyAssessment>("internal")?;

    let mut second = ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("second-owner")?;
    let local = second.assessment::<ConsistencyAssessment>("internal")?;
    assert!(matches!(
        second.input_port("subject", &foreign),
        Err(ApplicationWorkflowAuthoringDenial::ForeignComponentNode)
    ));
    assert!(matches!(
        second.control(
            &foreign,
            ApplicationWorkflowControlOutcome::Completed,
            &local,
        ),
        Err(ApplicationWorkflowAuthoringDenial::ForeignComponentNode)
    ));
    Ok(())
}

#[test]
fn component_port_identity_is_unique_within_the_component() -> Result<(), Box<dyn std::error::Error>>
{
    let mut component = ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("unique-ports")?;
    let internal = component.assessment::<ConsistencyAssessment>("internal")?;
    component.input_port("subject", &internal)?;
    assert!(matches!(
        component.output_port("subject", &internal),
        Err(ApplicationWorkflowAuthoringDenial::DuplicateComponentPort(identity))
            if identity == "subject"
    ));
    Ok(())
}

fn command_definition(
) -> Result<ValidatedWorkflowDefinition<ReviewedChange>, Box<dyn std::error::Error>> {
    use ApplicationWorkflowAuthoringCommand as Command;
    use ApplicationWorkflowControlOutcome::{
        Approved, Completed, EvidenceFailed, EvidenceSatisfied, Rejected,
    };
    use ApplicationWorkflowDataFlow::{
        ApprovalAuthority, AssessmentEvidence, AssessmentSubject, JoinedEvidence, OperationInput,
        ProposalSubject,
    };
    let commands = vec![
        Command::operation::<ReviewedChange, ProposeChange>("propose", false)?,
        Command::assessment::<ReviewedChange, ConsistencyAssessment>("checks/consistency")?,
        Command::assessment::<ReviewedChange, ComplianceAssessment>("checks/compliance")?,
        Command::evidence_join(
            "checks/evidence",
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )?,
        Command::approval::<ReviewedChange, ChangeApprover>("approval")?,
        Command::operation::<ReviewedChange, ApplyChange>("apply", true)?,
        Command::terminal("completed")?,
        Command::terminal("rejected")?,
        Command::start("propose")?,
        Command::control("propose", Completed, "checks/consistency")?,
        Command::control("checks/consistency", Completed, "checks/compliance")?,
        Command::control("checks/compliance", Completed, "checks/evidence")?,
        Command::control("checks/evidence", EvidenceSatisfied, "approval")?,
        Command::control("checks/evidence", EvidenceFailed, "rejected")?,
        Command::control("approval", Approved, "apply")?,
        Command::control("approval", Rejected, "rejected")?,
        Command::control("apply", Completed, "completed")?,
        Command::data("propose", AssessmentSubject, "checks/consistency")?,
        Command::data("propose", AssessmentSubject, "checks/compliance")?,
        Command::data("checks/consistency", AssessmentEvidence, "checks/evidence")?,
        Command::data("checks/compliance", AssessmentEvidence, "checks/evidence")?,
        Command::data("propose", ProposalSubject, "approval")?,
        Command::data("checks/evidence", JoinedEvidence, "approval")?,
        Command::data("approval", ApprovalAuthority, "apply")?,
        Command::data("propose", OperationInput, "apply")?,
    ];
    Ok(ApplicationWorkflowCommandAdapter::author::<ReviewedChange>(
        "reviewed-change",
        limits(),
        commands,
    )?
    .validate()?)
}
