//! The example's reusable review, authored once as a component.

use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowAssessmentNode, ApplicationWorkflowComponentBuilder,
    ApplicationWorkflowComponentInputPort, ApplicationWorkflowComponentOutputPort,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowEvidenceJoinNode,
    ApplicationWorkflowEvidenceJoinPolicy, AuthoredWorkflowComponent,
};

use crate::bounded_dimension_model::{
    schema::PartDimensionQuery, workflow::ReviewedGeometryWorkflow,
};

/// One review, authored once: two assessments of the proposed geometry
/// joined into the evidence an approval reads.
pub(crate) struct Review {
    pub(crate) component: AuthoredWorkflowComponent<ReviewedGeometryWorkflow>,
    pub(crate) structural: ApplicationWorkflowComponentInputPort<ApplicationWorkflowAssessmentNode>,
    pub(crate) independent:
        ApplicationWorkflowComponentInputPort<ApplicationWorkflowAssessmentNode>,
    pub(crate) evidence:
        ApplicationWorkflowComponentOutputPort<ApplicationWorkflowEvidenceJoinNode>,
}

pub(crate) fn review() -> Review {
    let mut review =
        ApplicationWorkflowComponentBuilder::<ReviewedGeometryWorkflow>::new("geometry-review")
            .expect("the component identity is valid");
    let structural = review
        .assessment::<PartDimensionQuery>("structural")
        .expect("the structural assessment is valid");
    let independent = review
        .assessment::<PartDimensionQuery>("independent")
        .expect("the independent assessment is valid");
    let evidence = review
        .evidence_join(
            "evidence",
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .expect("the join is valid");
    review
        .control(
            &structural,
            ApplicationWorkflowControlOutcome::Completed,
            &independent,
        )
        .expect("the structural review hands on")
        .control(
            &independent,
            ApplicationWorkflowControlOutcome::Completed,
            &evidence,
        )
        .expect("the independent review hands on")
        .assessment_evidence(&structural, &evidence)
        .expect("structural evidence joins")
        .assessment_evidence(&independent, &evidence)
        .expect("independent evidence joins");
    Review {
        structural: review
            .input_port("structural-subject", &structural)
            .expect("the structural port is valid"),
        independent: review
            .input_port("independent-subject", &independent)
            .expect("the independent port is valid"),
        evidence: review
            .output_port("reviewed-evidence", &evidence)
            .expect("the evidence port is valid"),
        component: review.finish().expect("the review component closes"),
    }
}
