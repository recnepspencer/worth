use super::*;

#[test]
fn lineage_identity_is_not_part_of_canonical_content_identity(
) -> Result<(), Box<dyn std::error::Error>> {
    let first = primitive_definition_named("reviewed-geometry-a")?;
    let second = primitive_definition_named("reviewed-geometry-b")?;
    assert_ne!(first.identity(), second.identity());
    assert_eq!(first.content_identity(), second.content_identity());
    Ok(())
}

#[test]
fn canonical_node_fields_are_unambiguously_framed() -> Result<(), Box<dyn std::error::Error>> {
    fn definition<Operation>(
        identity: &str,
    ) -> Result<ValidatedWorkflowDefinition<ReviewedGeometry>, Box<dyn std::error::Error>>
    where
        Operation: ApplicationOperationMarkerIdentity<TestSchema> + 'static,
    {
        let mut builder =
            ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(identity, limits())?;
        let operation = builder.operation::<Operation>("operation", false)?;
        let terminal = builder.terminal("terminal")?;
        builder.start(&operation).control(
            &operation,
            ApplicationWorkflowControlOutcome::Completed,
            &terminal,
        );
        Ok(builder.finish()?.validate()?)
    }

    let first = definition::<CollisionOperationAb>("collision-a")?;
    let second = definition::<CollisionOperationA>("collision-b")?;
    assert_ne!(first.content_identity(), second.content_identity());
    Ok(())
}

#[test]
fn primitive_form_has_the_independently_expected_expanded_graph() {
    let definition = primitive_definition().expect("the reference graph validates");
    let identities = definition
        .nodes()
        .iter()
        .map(|node| node.identity().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        identities,
        [
            "apply",
            "approval",
            "checks/evidence",
            "checks/manufacturability",
            "checks/structural",
            "completed",
            "propose",
            "rejected",
        ]
    );
    let kind = |identity: &str| {
        definition
            .nodes()
            .iter()
            .find(|node| node.identity().as_str() == identity)
            .expect("the independently expected node exists")
            .kind()
    };
    assert!(matches!(
        kind("propose"),
        ApplicationWorkflowNodeKind::Operation {
            requires_workflow_authority: false,
            ..
        }
    ));
    assert!(matches!(
        kind("checks/structural"),
        ApplicationWorkflowNodeKind::Assessment(_)
    ));
    assert!(matches!(
        kind("checks/manufacturability"),
        ApplicationWorkflowNodeKind::Assessment(_)
    ));
    assert!(matches!(
        kind("checks/evidence"),
        ApplicationWorkflowNodeKind::EvidenceJoin
    ));
    assert!(matches!(
        kind("approval"),
        ApplicationWorkflowNodeKind::Approval(_)
    ));
    assert!(matches!(
        kind("apply"),
        ApplicationWorkflowNodeKind::Operation {
            requires_workflow_authority: true,
            ..
        }
    ));
    assert!(matches!(
        kind("completed"),
        ApplicationWorkflowNodeKind::Terminal
    ));
    assert!(matches!(
        kind("rejected"),
        ApplicationWorkflowNodeKind::Terminal
    ));

    use ApplicationWorkflowConnectionKind::{Control, Data};
    use ApplicationWorkflowControlOutcome::{Approved, Completed, Rejected};
    use ApplicationWorkflowDataFlow::{
        ApprovalAuthority, AssessmentEvidence, AssessmentSubject, JoinedEvidence, OperationInput,
        ProposalSubject,
    };
    let connections = definition
        .connections()
        .iter()
        .map(|connection| {
            (
                connection.source().as_str(),
                connection.kind(),
                connection.target().as_str(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        connections,
        std::collections::BTreeSet::from([
            ("propose", Control(Completed), "checks/structural"),
            (
                "checks/structural",
                Control(Completed),
                "checks/manufacturability",
            ),
            (
                "checks/manufacturability",
                Control(Completed),
                "checks/evidence",
            ),
            ("checks/evidence", Control(Completed), "approval"),
            ("approval", Control(Approved), "apply"),
            ("approval", Control(Rejected), "rejected"),
            ("apply", Control(Completed), "completed"),
            ("propose", Data(AssessmentSubject), "checks/structural"),
            (
                "propose",
                Data(AssessmentSubject),
                "checks/manufacturability",
            ),
            (
                "checks/structural",
                Data(AssessmentEvidence),
                "checks/evidence",
            ),
            (
                "checks/manufacturability",
                Data(AssessmentEvidence),
                "checks/evidence",
            ),
            ("propose", Data(ProposalSubject), "approval"),
            ("checks/evidence", Data(JoinedEvidence), "approval"),
            ("approval", Data(ApprovalAuthority), "apply"),
            ("propose", Data(OperationInput), "apply"),
        ])
    );
}
