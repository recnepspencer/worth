use bank_domain::queries::PaymentDetailQuery;
use bank_domain::schema::{
    ApprovePaymentOperation, ApprovedBusinessPaymentApproval,
    ApprovedBusinessPaymentAuthoringOperation, ApprovedBusinessPaymentWorkflow,
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationWorkflowAuthoringDenial, ApplicationWorkflowComponentLimits,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
    ApplicationWorkflowDefinitionLimits, ApplicationWorkflowEvidenceJoinPolicy,
    ApplicationWorkflowValidationDenial, ValidatedWorkflowDefinition,
};

pub fn approved_business_payment_definition() -> Result<
    ValidatedWorkflowDefinition<ApprovedBusinessPaymentWorkflow>,
    ApprovedBusinessPaymentDefinitionDenial,
> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ApprovedBusinessPaymentWorkflow>::new(
        "approved-business-payment",
        ApplicationWorkflowDefinitionLimits::new(
            8,
            16,
            2,
            ApplicationWorkflowComponentLimits::new(8, 2, 16, 32, 32).unwrap(),
            8 * 1_024,
        )
        .expect("approved-payment limits are nonzero"),
    )
    .expect("approved-payment identity is valid");
    let propose =
        builder.operation::<ApprovedBusinessPaymentAuthoringOperation>("propose", false)?;
    let payment = builder.assessment::<PaymentDetailQuery>("review/payment")?;
    let independent = builder.assessment::<PaymentDetailQuery>("review/independent")?;
    let evidence = builder.evidence_join(
        "review/evidence",
        ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
    )?;
    let approval = builder.approval::<ApprovedBusinessPaymentApproval>("approval")?;
    let apply = builder.operation::<ApprovePaymentOperation>("apply", true)?;
    let completed = builder.terminal("completed")?;
    let rejected = builder.terminal("rejected")?;

    builder
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &payment,
        )
        .control(
            &payment,
            ApplicationWorkflowControlOutcome::Completed,
            &independent,
        )
        .control(
            &independent,
            ApplicationWorkflowControlOutcome::Completed,
            &evidence,
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
        .proposal_for_assessment(&propose, &payment)
        .proposal_for_assessment(&propose, &independent)
        .assessment_evidence(&payment, &evidence)
        .assessment_evidence(&independent, &evidence)
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);

    Ok(builder.finish()?.validate()?)
}

#[derive(Debug)]
pub enum ApprovedBusinessPaymentDefinitionDenial {
    Authoring(ApplicationWorkflowAuthoringDenial),
    Validation(ApplicationWorkflowValidationDenial),
}

impl From<ApplicationWorkflowAuthoringDenial> for ApprovedBusinessPaymentDefinitionDenial {
    fn from(denial: ApplicationWorkflowAuthoringDenial) -> Self {
        Self::Authoring(denial)
    }
}

impl From<ApplicationWorkflowValidationDenial> for ApprovedBusinessPaymentDefinitionDenial {
    fn from(denial: ApplicationWorkflowValidationDenial) -> Self {
        Self::Validation(denial)
    }
}

#[cfg(test)]
mod tests {
    use bank_domain::schema::{ApprovedBusinessPaymentAuthoringBinding, BankSchema};
    use worth_query_host::facade::declaration::application_program::{
        ApplicationWorkflowConnectionKind, ApplicationWorkflowDataFlow, ApplicationWorkflowNodeKind,
    };
    use worth_query_host::facade::declaration::{
        application_operation::ApplicationMutationBinding,
        application_schema::ApplicationSchemaMember,
    };

    #[test]
    fn approved_business_payment_uses_the_real_payment_operation() {
        let definition = super::approved_business_payment_definition()
            .expect("the approved-payment definition validates");
        assert_eq!(definition.nodes().len(), 8);
        assert_eq!(definition.connections().len(), 16);
        assert!(definition.nodes().iter().any(|node| {
            node.identity().as_str() == "apply"
                && matches!(
                    node.kind(),
                    ApplicationWorkflowNodeKind::Operation {
                        operation,
                        requires_workflow_authority: true,
                    } if operation.identifier() == "ApprovePaymentOperation"
                )
        }));
        assert!(definition.connections().iter().any(|connection| matches!(
            connection.kind(),
            ApplicationWorkflowConnectionKind::Data(ApplicationWorkflowDataFlow::ApprovalAuthority)
        )));
    }

    #[test]
    fn bank_payments_contribution_owns_the_workflow_control_binding() {
        let declaration = BankSchema::declaration().expect("the Bank schema declares");
        let erased = declaration.erased();
        let payments = declaration
            .contributions()
            .iter()
            .find(|contribution| contribution.identity().as_str() == "worth.bank.payments.v1")
            .expect("the Bank payments contribution is present");
        assert!(payments.member_ordinals().iter().any(|ordinal| {
            matches!(
                &erased.members()[*ordinal as usize],
                ApplicationSchemaMember::ApplicationMutation { description }
                    if description.binding_identity().as_str()
                        == ApprovedBusinessPaymentAuthoringBinding::IDENTITY
            )
        }));
        let program = super::super::validated_bank_application()
            .expect("the Bank application program validates");
        assert!(program.actions().iter().any(|action| {
            action.binding() == ApprovedBusinessPaymentAuthoringBinding::IDENTITY
        }));
    }
}
