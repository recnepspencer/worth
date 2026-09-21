use crate::{
    application_capability::ApplicationCapabilityMarkerIdentity,
    application_query::ApplicationQueryMarkerIdentity,
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationSchemaDeclaration,
        ApplicationSchemaDeclarationDenial, ApplicationStructuredValueBinding,
        ApplicationValueValidationDenial,
    },
    portable_identity::WorthQueryPortableType,
};

use super::*;

mod authoring;
mod condition;
mod retry;
mod shape;
mod validation;

struct TestSchema;
struct ReviewedGeometry;
struct ProposalInput;
struct ProposalInputBinding;
struct AssessmentInput;
struct AssessmentInputBinding;
struct AssessmentResult;
struct AssessmentResultBinding;
struct BoolResultBinding;
struct ProposeChange;
struct ApplyChange;
struct StructuralAssessment;
struct ManufacturabilityAssessment;
struct StructuralCondition;
struct ManufacturabilityCondition;
struct GeometryApprover;
struct CollisionInputC;
struct CollisionInputCBinding;
struct CollisionInputBc;
struct CollisionInputBcBinding;
struct CollisionOperationAb;
struct CollisionOperationA;

impl ApplicationSchema for TestSchema {
    const OWNER: &'static str = "worth.query.tests";
    const NAME: &'static str = "workflow";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        unreachable!("workflow declaration tests do not install a schema")
    }
}

impl ApplicationWorkflowSpec for ReviewedGeometry {
    type Schema = TestSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.query.tests.reviewed-geometry.v1");
}

macro_rules! value_binding {
    ($binding:ty, $value:ty, $identity:literal) => {
        impl ApplicationStructuredValueBinding for $binding {
            type Value = $value;
            const IDENTITY_NAME: &'static str = $identity;

            fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
                Ok(())
            }
        }
    };
}

value_binding!(
    ProposalInputBinding,
    ProposalInput,
    "worth.query.tests.workflow.proposal-input.v1"
);
value_binding!(CollisionInputCBinding, CollisionInputC, "c");
value_binding!(CollisionInputBcBinding, CollisionInputBc, "b:c");
value_binding!(
    AssessmentInputBinding,
    AssessmentInput,
    "worth.query.tests.workflow.assessment-input.v1"
);
value_binding!(
    AssessmentResultBinding,
    AssessmentResult,
    "worth.query.tests.workflow.assessment-result.v1"
);
value_binding!(
    BoolResultBinding,
    bool,
    "worth.query.tests.workflow.bool-result.v1"
);

impl ApplicationOperationMarkerIdentity<TestSchema> for ProposeChange {
    type InputBinding = ProposalInputBinding;
    const IDENTIFIER: &'static str = "worth.query.tests.workflow.propose.v1";
}

impl ApplicationOperationMarkerIdentity<TestSchema> for ApplyChange {
    type InputBinding = ProposalInputBinding;
    const IDENTIFIER: &'static str = "worth.query.tests.workflow.apply.v1";
}

impl ApplicationOperationMarkerIdentity<TestSchema> for CollisionOperationAb {
    type InputBinding = CollisionInputCBinding;
    const IDENTIFIER: &'static str = "a:b";
}

impl ApplicationOperationMarkerIdentity<TestSchema> for CollisionOperationA {
    type InputBinding = CollisionInputBcBinding;
    const IDENTIFIER: &'static str = "a";
}

macro_rules! assessment_query {
    ($query:ty, $identity:literal) => {
        impl ApplicationQueryMarkerIdentity<TestSchema> for $query {
            type ParameterBinding = AssessmentInputBinding;
            type ResultBinding = AssessmentResultBinding;
            type Scope = ();
            const IDENTIFIER: &'static str = $identity;
            const QUERY_TYPE_NAME: &'static str = concat!($identity, ".query-type");
            const SCOPE_TYPE_NAME: &'static str = "worth.rust.unit";
        }
    };
}

assessment_query!(
    StructuralAssessment,
    "worth.query.tests.workflow.structural-assessment.v1"
);

assessment_query!(
    ManufacturabilityAssessment,
    "worth.query.tests.workflow.manufacturability-assessment.v1"
);

macro_rules! condition_query {
    ($query:ty, $identity:literal) => {
        impl ApplicationQueryMarkerIdentity<TestSchema> for $query {
            type ParameterBinding = AssessmentInputBinding;
            type ResultBinding = BoolResultBinding;
            type Scope = ();
            const IDENTIFIER: &'static str = $identity;
            const QUERY_TYPE_NAME: &'static str = concat!($identity, ".query-type");
            const SCOPE_TYPE_NAME: &'static str = "worth.rust.unit";
        }
    };
}
condition_query!(
    StructuralCondition,
    "worth.query.tests.workflow.structural-condition.v1"
);
condition_query!(
    ManufacturabilityCondition,
    "worth.query.tests.workflow.manufacturability-condition.v1"
);

impl WorthQueryPortableType for GeometryApprover {
    const PORTABLE_TYPE_NAME: &'static str = "worth.query.tests.workflow.geometry-approver.v1";
}

impl ApplicationCapabilityMarkerIdentity for GeometryApprover {
    type Schema = TestSchema;
    const IDENTIFIER: &'static str = "worth.query.tests.workflow.geometry-approval.v1";
}

fn limits() -> ApplicationWorkflowDefinitionLimits {
    ApplicationWorkflowDefinitionLimits::new(32, 64, 4, 4, 64 * 1024)
        .expect("test limits are nonzero")
}

fn primitive_definition(
) -> Result<ValidatedWorkflowDefinition<ReviewedGeometry>, Box<dyn std::error::Error>> {
    primitive_definition_named("reviewed-geometry")
}

fn primitive_definition_named(
    identity: &str,
) -> Result<ValidatedWorkflowDefinition<ReviewedGeometry>, Box<dyn std::error::Error>> {
    primitive_definition_named_with_policy(
        identity,
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )
}

fn primitive_definition_named_with_policy(
    identity: &str,
    policy: ApplicationWorkflowEvidenceJoinPolicy,
) -> Result<ValidatedWorkflowDefinition<ReviewedGeometry>, Box<dyn std::error::Error>> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(identity, limits())?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let structural = builder.assessment::<StructuralAssessment>("checks/structural")?;
    let manufacturability =
        builder.assessment::<ManufacturabilityAssessment>("checks/manufacturability")?;
    let evidence = builder.evidence_join("checks/evidence", policy)?;
    let approval = builder.approval::<GeometryApprover>("approval")?;
    let apply = builder.operation::<ApplyChange>("apply", true)?;
    let completed = builder.terminal("completed")?;
    let rejected = builder.terminal("rejected")?;
    connect_definition(
        &mut builder,
        &propose,
        &structural,
        &manufacturability,
        &evidence,
        &approval,
        &apply,
        &completed,
        &rejected,
    );
    Ok(builder.finish()?.validate()?)
}

fn component_definition(
) -> Result<ValidatedWorkflowDefinition<ReviewedGeometry>, Box<dyn std::error::Error>> {
    let mut component =
        ApplicationWorkflowComponentBuilder::<ReviewedGeometry>::new("required-geometry-review")?;
    let structural = component.assessment::<StructuralAssessment>("structural")?;
    let manufacturability =
        component.assessment::<ManufacturabilityAssessment>("manufacturability")?;
    let evidence = component.evidence_join(
        "evidence",
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )?;
    component.control(
        &structural,
        ApplicationWorkflowControlOutcome::Completed,
        &manufacturability,
    )?;
    component.control(
        &manufacturability,
        ApplicationWorkflowControlOutcome::Completed,
        &evidence,
    )?;
    component.assessment_evidence(&structural, &evidence)?;
    component.assessment_evidence(&manufacturability, &evidence)?;
    let structural_input = component.input_port("structural-subject", &structural)?;
    let manufacturability_input =
        component.input_port("manufacturability-subject", &manufacturability)?;
    let evidence_output = component.output_port("reviewed-evidence", &evidence)?;
    let component = component.finish()?;

    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometry>::new(
        "reviewed-geometry",
        limits(),
    )?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let approval = builder.approval::<GeometryApprover>("approval")?;
    let apply = builder.operation::<ApplyChange>("apply", true)?;
    let completed = builder.terminal("completed")?;
    let rejected = builder.terminal("rejected")?;
    let checks = builder.expand_component("checks", &component)?;
    let structural = checks.input(&structural_input)?;
    let manufacturability = checks.input(&manufacturability_input)?;
    let evidence = checks.output(&evidence_output)?;
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &structural,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &approval,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &rejected,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &apply,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Rejected,
            &rejected,
        )
        .control(
            &apply,
            ApplicationWorkflowControlOutcome::Completed,
            &completed,
        )
        .proposal_for_assessment(&propose, &structural)
        .proposal_for_assessment(&propose, &manufacturability)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);
    Ok(builder.finish()?.validate()?)
}

#[allow(clippy::too_many_arguments)]
fn connect_definition(
    builder: &mut ApplicationWorkflowDefinitionBuilder<ReviewedGeometry>,
    propose: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
    structural: &ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
    manufacturability: &ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
    evidence: &ApplicationWorkflowNodeRef<ApplicationWorkflowEvidenceJoinNode>,
    approval: &ApplicationWorkflowNodeRef<ApplicationWorkflowApprovalNode>,
    apply: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
    completed: &ApplicationWorkflowNodeRef<ApplicationWorkflowTerminalNode>,
    rejected: &ApplicationWorkflowNodeRef<ApplicationWorkflowTerminalNode>,
) {
    builder
        .start(propose)
        .control(
            propose,
            ApplicationWorkflowControlOutcome::Completed,
            structural,
        )
        .control(
            structural,
            ApplicationWorkflowControlOutcome::Completed,
            manufacturability,
        )
        .control(
            manufacturability,
            ApplicationWorkflowControlOutcome::Completed,
            evidence,
        )
        .control(
            evidence,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            approval,
        )
        .control(
            evidence,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            rejected,
        )
        .control(approval, ApplicationWorkflowControlOutcome::Approved, apply)
        .control(
            approval,
            ApplicationWorkflowControlOutcome::Rejected,
            rejected,
        )
        .control(
            apply,
            ApplicationWorkflowControlOutcome::Completed,
            completed,
        )
        .proposal_for_assessment(propose, structural)
        .proposal_for_assessment(propose, manufacturability)
        .assessment_evidence(structural, evidence)
        .assessment_evidence(manufacturability, evidence)
        .proposal_for_approval(propose, approval)
        .joined_evidence(evidence, approval)
        .approval_authority(approval, apply)
        .operation_input(propose, apply);
}
