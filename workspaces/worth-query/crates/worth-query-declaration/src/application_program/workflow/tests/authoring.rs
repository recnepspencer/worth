use super::*;

#[test]
fn primitive_component_macro_and_command_authoring_are_canonical_equivalents(
) -> Result<(), Box<dyn std::error::Error>> {
    let primitive = primitive_definition()?;
    let component = component_definition()?;
    let macro_authored = crate::worth_query_workflow! {
        spec: ReviewedGeometry;
        identity: "reviewed-geometry";
        limits: limits();
        build: |builder| {
            let propose = builder.operation::<ProposeChange>("propose", false)?;
            let structural = builder.assessment::<StructuralAssessment>("checks/structural")?;
            let manufacturability = builder.assessment::<ManufacturabilityAssessment>("checks/manufacturability")?;
            let evidence = builder.evidence_join("checks/evidence")?;
            let approval = builder.approval::<GeometryApprover>("approval")?;
            let apply = builder.operation::<ApplyChange>("apply", true)?;
            let completed = builder.terminal("completed")?;
            let rejected = builder.terminal("rejected")?;
            connect_definition(&mut builder, &propose, &structural, &manufacturability, &evidence, &approval, &apply, &completed, &rejected);
        }
    }?.validate()?;
    let command = command_definition()?;
    assert_eq!(primitive.content_identity(), component.content_identity());
    assert_eq!(
        primitive.content_identity(),
        macro_authored.content_identity()
    );
    assert_eq!(primitive.content_identity(), command.content_identity());
    assert_eq!(primitive.nodes(), command.nodes());
    assert_eq!(primitive.connections(), component.connections());
    Ok(())
}

fn command_definition(
) -> Result<ValidatedWorkflowDefinition<ReviewedGeometry>, Box<dyn std::error::Error>> {
    use ApplicationWorkflowAuthoringCommand as Command;
    use ApplicationWorkflowControlOutcome::{Approved, Completed, Rejected};
    use ApplicationWorkflowDataFlow::{
        ApprovalAuthority, AssessmentEvidence, AssessmentSubject, JoinedEvidence, OperationInput,
        ProposalSubject,
    };
    let commands = vec![
        Command::operation::<ReviewedGeometry, ProposeChange>("propose", false)?,
        Command::assessment::<ReviewedGeometry, StructuralAssessment>("checks/structural")?,
        Command::assessment::<ReviewedGeometry, ManufacturabilityAssessment>(
            "checks/manufacturability",
        )?,
        Command::evidence_join("checks/evidence")?,
        Command::approval::<ReviewedGeometry, GeometryApprover>("approval")?,
        Command::operation::<ReviewedGeometry, ApplyChange>("apply", true)?,
        Command::terminal("completed")?,
        Command::terminal("rejected")?,
        Command::start("propose")?,
        Command::control("propose", Completed, "checks/structural")?,
        Command::control("checks/structural", Completed, "checks/manufacturability")?,
        Command::control("checks/manufacturability", Completed, "checks/evidence")?,
        Command::control("checks/evidence", Completed, "approval")?,
        Command::control("approval", Approved, "apply")?,
        Command::control("approval", Rejected, "rejected")?,
        Command::control("apply", Completed, "completed")?,
        Command::data("propose", AssessmentSubject, "checks/structural")?,
        Command::data("propose", AssessmentSubject, "checks/manufacturability")?,
        Command::data("checks/structural", AssessmentEvidence, "checks/evidence")?,
        Command::data(
            "checks/manufacturability",
            AssessmentEvidence,
            "checks/evidence",
        )?,
        Command::data("propose", ProposalSubject, "approval")?,
        Command::data("checks/evidence", JoinedEvidence, "approval")?,
        Command::data("approval", ApprovalAuthority, "apply")?,
        Command::data("propose", OperationInput, "apply")?,
    ];
    Ok(
        ApplicationWorkflowCommandAdapter::author::<ReviewedGeometry>(
            "reviewed-geometry",
            limits(),
            commands,
        )?
        .validate()?,
    )
}
