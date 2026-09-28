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
mod content_identity;
mod resource_limits;
mod retry;
mod scale;
mod sequence;
mod shape;
mod subject_selector;
mod validation;

struct TestSchema;
struct ReviewedChange;
struct ProposalInput;
struct ProposalInputBinding;
struct AssessmentInput;
struct AssessmentInputBinding;
struct AssessmentResult;
struct AssessmentResultBinding;
struct BoolResultBinding;
struct ProposeChange;
struct ApplyChange;
struct ConsistencyAssessment;
struct ComplianceAssessment;
struct ConsistencyCondition;
struct ComplianceCondition;
struct ReviewCount;
struct ReviewCountBinding;
struct ChangeApprover;
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

impl ApplicationWorkflowSpec for ReviewedChange {
    type Schema = TestSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.query.tests.reviewed-change.v1");
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
value_binding!(
    ReviewCountBinding,
    Option<i64>,
    "worth.query.tests.workflow.review-count.v1"
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
    ConsistencyAssessment,
    "worth.query.tests.workflow.consistency-assessment.v1"
);

assessment_query!(
    ComplianceAssessment,
    "worth.query.tests.workflow.compliance-assessment.v1"
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
    ConsistencyCondition,
    "worth.query.tests.workflow.consistency-condition.v1"
);
condition_query!(
    ComplianceCondition,
    "worth.query.tests.workflow.compliance-condition.v1"
);

impl ApplicationQueryMarkerIdentity<TestSchema> for ReviewCount {
    type ParameterBinding = AssessmentInputBinding;
    type ResultBinding = ReviewCountBinding;
    type Scope = ();
    const IDENTIFIER: &'static str = "worth.query.tests.workflow.review-count.v1";
    const QUERY_TYPE_NAME: &'static str = "worth.query.tests.workflow.review-count.v1.query-type";
    const SCOPE_TYPE_NAME: &'static str = "worth.rust.unit";
}

impl WorthQueryPortableType for ChangeApprover {
    const PORTABLE_TYPE_NAME: &'static str = "worth.query.tests.workflow.change-approver.v1";
}

impl ApplicationCapabilityMarkerIdentity for ChangeApprover {
    type Schema = TestSchema;
    const IDENTIFIER: &'static str = "worth.query.tests.workflow.change-approval.v1";
}

fn limits() -> ApplicationWorkflowDefinitionLimits {
    ApplicationWorkflowDefinitionLimits::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
    )
    .expect("test limits are nonzero")
}

fn primitive_definition(
) -> Result<ValidatedWorkflowDefinition<ReviewedChange>, Box<dyn std::error::Error>> {
    primitive_definition_named("reviewed-change")
}

fn primitive_definition_named(
    identity: &str,
) -> Result<ValidatedWorkflowDefinition<ReviewedChange>, Box<dyn std::error::Error>> {
    primitive_definition_named_with_policy(
        identity,
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )
}

fn primitive_definition_named_with_policy(
    identity: &str,
    policy: ApplicationWorkflowEvidenceJoinPolicy,
) -> Result<ValidatedWorkflowDefinition<ReviewedChange>, Box<dyn std::error::Error>> {
    primitive_definition_with_limits(identity, policy, limits())
}

fn primitive_definition_with_limits(
    identity: &str,
    policy: ApplicationWorkflowEvidenceJoinPolicy,
    limits: ApplicationWorkflowDefinitionLimits,
) -> Result<ValidatedWorkflowDefinition<ReviewedChange>, Box<dyn std::error::Error>> {
    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new(identity, limits)?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let consistency = builder.assessment::<ConsistencyAssessment>("checks/consistency")?;
    let compliance = builder.assessment::<ComplianceAssessment>("checks/compliance")?;
    let evidence = builder.evidence_join("checks/evidence", policy)?;
    let approval = builder.approval::<ChangeApprover>("approval")?;
    let apply = builder.operation::<ApplyChange>("apply", true)?;
    let completed = builder.terminal("completed")?;
    let rejected = builder.terminal("rejected")?;
    connect_definition(
        &mut builder,
        &propose,
        &consistency,
        &compliance,
        &evidence,
        &approval,
        &apply,
        &completed,
        &rejected,
    );
    Ok(builder.finish()?.validate()?)
}

fn component_definition(
) -> Result<ValidatedWorkflowDefinition<ReviewedChange>, Box<dyn std::error::Error>> {
    let mut component =
        ApplicationWorkflowComponentBuilder::<ReviewedChange>::new("required-change-review")?;
    let consistency = component.assessment::<ConsistencyAssessment>("consistency")?;
    let compliance = component.assessment::<ComplianceAssessment>("compliance")?;
    let evidence = component.evidence_join(
        "evidence",
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )?;
    component.control(
        &consistency,
        ApplicationWorkflowControlOutcome::Completed,
        &compliance,
    )?;
    component.control(
        &compliance,
        ApplicationWorkflowControlOutcome::Completed,
        &evidence,
    )?;
    component.assessment_evidence(&consistency, &evidence)?;
    component.assessment_evidence(&compliance, &evidence)?;
    let consistency_input = component.input_port("consistency-subject", &consistency)?;
    let compliance_input = component.input_port("compliance-subject", &compliance)?;
    let evidence_output = component.output_port("reviewed-evidence", &evidence)?;
    let component = component.finish()?;

    let mut builder =
        ApplicationWorkflowDefinitionBuilder::<ReviewedChange>::new("reviewed-change", limits())?;
    let propose = builder.operation::<ProposeChange>("propose", false)?;
    let approval = builder.approval::<ChangeApprover>("approval")?;
    let apply = builder.operation::<ApplyChange>("apply", true)?;
    let completed = builder.terminal("completed")?;
    let rejected = builder.terminal("rejected")?;
    let checks = builder.expand_component("checks", &component)?;
    let consistency = checks.input(&consistency_input)?;
    let compliance = checks.input(&compliance_input)?;
    let evidence = checks.output(&evidence_output)?;
    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &consistency,
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
        .proposal_for_assessment(&propose, &consistency)
        .proposal_for_assessment(&propose, &compliance)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);
    Ok(builder.finish()?.validate()?)
}

#[allow(clippy::too_many_arguments)]
fn connect_definition(
    builder: &mut ApplicationWorkflowDefinitionBuilder<ReviewedChange>,
    propose: &ApplicationWorkflowNodeRef<ApplicationWorkflowOperationNode>,
    consistency: &ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
    compliance: &ApplicationWorkflowNodeRef<ApplicationWorkflowAssessmentNode>,
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
            consistency,
        )
        .control(
            consistency,
            ApplicationWorkflowControlOutcome::Completed,
            compliance,
        )
        .control(
            compliance,
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
        .proposal_for_assessment(propose, consistency)
        .proposal_for_assessment(propose, compliance)
        .assessment_evidence(consistency, evidence)
        .assessment_evidence(compliance, evidence)
        .proposal_for_approval(propose, approval)
        .joined_evidence(evidence, approval)
        .approval_authority(approval, apply)
        .operation_input(propose, apply);
}
