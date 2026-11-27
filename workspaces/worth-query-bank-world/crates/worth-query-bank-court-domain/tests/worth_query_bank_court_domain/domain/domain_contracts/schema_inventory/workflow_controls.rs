//! Bank's macro-declared workflow controls keep the hand-written wire identities.
use bank_domain::model::{BankPrincipalId, PaymentId};
use bank_domain::schema::{
    ApprovePayment, ApprovedBusinessPaymentAdvance, ApprovedBusinessPaymentAdvanceBinding,
    ApprovedBusinessPaymentApproval, ApprovedBusinessPaymentApprovalBinding,
    ApprovedBusinessPaymentAuthoring, ApprovedBusinessPaymentAuthoringBinding,
    ApprovedBusinessPaymentControlContext, ApprovedBusinessPaymentInstanceStart,
    ApprovedBusinessPaymentInstanceStartBinding, BankSchema,
};
use worth_query_decl::facade::application_schema::ApplicationValue as AspectValue;
use worth_query_bank_testkit_domain::facade::domain_contracts::schema_inventory::workflow_controls::HAND_WRITTEN_CONTROL_IDENTITIES;
use worth_query_decl::facade::{
    application_capability::{
        ApplicationCapabilityContextMarkerIdentity, ApplicationCapabilityMarkerIdentity,
        ApplicationCapabilityRequest,
    },
    application_operation::ApplicationMutationBinding,
    portable_identity::WorthQueryPortableType,
};

fn identities<Binding, Capability>() -> [&'static str; 5]
where
    Binding: ApplicationMutationBinding<BankSchema>,
    Capability: ApplicationCapabilityMarkerIdentity + WorthQueryPortableType,
{
    assert!(Binding::WORKFLOW_CONTROL && Binding::REQUIRES_APPLICATION_PROGRAM);
    [
        Capability::IDENTIFIER,
        Capability::PORTABLE_TYPE_NAME,
        Binding::IDENTITY,
        Binding::HANDLER_IDENTITY,
        Binding::IDEMPOTENCY_IDENTITY,
    ]
}

/// The projection exactly as the hand-written `capability_request` built it.
fn projected<Capability>(input: &ApprovePayment) -> (String, AspectValue, AspectValue, String)
where
    ApprovePayment: ApplicationCapabilityRequest<BankSchema, Capability>,
{
    let projection = match input.capability_request() {
        Ok(projection) => projection,
        Err(denial) => panic!("the control request must project: {denial:?}"),
    };
    (
        format!(
            "{}.{}.{}={:?}",
            projection.resource().entity(),
            projection.resource().aspect(),
            projection.resource().field(),
            projection.resource().value()
        ),
        projection.action().clone(),
        projection.purpose().clone(),
        projection.context_value().context_type().to_owned(),
    )
}

#[test]
fn macro_declared_bank_controls_keep_the_hand_written_wire_identities() {
    assert_eq!(
        [
            identities::<ApprovedBusinessPaymentApprovalBinding, ApprovedBusinessPaymentApproval>(),
            identities::<ApprovedBusinessPaymentAuthoringBinding, ApprovedBusinessPaymentAuthoring>(
            ),
            identities::<
                ApprovedBusinessPaymentInstanceStartBinding,
                ApprovedBusinessPaymentInstanceStart,
            >(),
            identities::<ApprovedBusinessPaymentAdvanceBinding, ApprovedBusinessPaymentAdvance>(),
        ],
        HAND_WRITTEN_CONTROL_IDENTITIES
    );
    assert_eq!(
        (
            <ApprovedBusinessPaymentControlContext as ApplicationCapabilityContextMarkerIdentity>::IDENTIFIER,
            <ApprovedBusinessPaymentControlContext as WorthQueryPortableType>::PORTABLE_TYPE_NAME,
        ),
        (
            "ApprovedBusinessPaymentControlContext",
            "worth.bank.approved-business-payment-control-context.v1"
        )
    );
    let (Some(payment), Some(approver)) = (PaymentId::new(41), BankPrincipalId::new(7)) else {
        panic!("fixture identifiers are valid");
    };
    let input = ApprovePayment { payment, approver };
    let action = AspectValue::String("manage-approved-business-payment-workflow".into());
    let purpose = AspectValue::String("business-payment-approval".into());
    for projection in [
        projected::<ApprovedBusinessPaymentApproval>(&input),
        projected::<ApprovedBusinessPaymentAuthoring>(&input),
        projected::<ApprovedBusinessPaymentInstanceStart>(&input),
        projected::<ApprovedBusinessPaymentAdvance>(&input),
    ] {
        assert!(
            projection.0.starts_with("PaymentIntent."),
            "{}",
            projection.0
        );
        assert!(
            projection.0.contains(".PaymentIdentityField="),
            "{}",
            projection.0
        );
        assert_eq!(
            (&projection.1, &projection.2, projection.3.as_str()),
            (
                &action,
                &purpose,
                "worth.bank.approved-business-payment-control-context.v1"
            )
        );
    }
}
