use bank_domain::model::BankPrincipalId;
use bank_domain::payments::BusinessPayment;
use bank_domain::schema::{
    ApprovedBusinessPaymentGrant, ApprovedBusinessPaymentGrantAction,
    ApprovedBusinessPaymentGrantDelegationLimit, ApprovedBusinessPaymentGrantGrantee,
    ApprovedBusinessPaymentGrantGrantor, ApprovedBusinessPaymentGrantNotAfter,
    ApprovedBusinessPaymentGrantNotBefore, ApprovedBusinessPaymentGrantPurpose,
    ApprovedBusinessPaymentGrantResource, ApprovedBusinessPaymentGrantStatus,
    ApprovedBusinessPaymentGrantWorkflow, BankSchema, Business,
    InitiateBusinessPaymentMutationBinding, PaymentIntent, Principal, PrincipalIdentityField,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, HandlerExecutionDenial, WorthQueryApplicationEntityKey,
};

use crate::graph_bootstrap::payment_workflow_grant_key;

type Writer<'candidate> =
    CandidateWriter<'candidate, BankSchema, InitiateBusinessPaymentMutationBinding>;
type Entity<Kind> =
    worth_query_host::facade::primary_graph::WorthQueryApplicationEffectEntity<BankSchema, Kind>;

/// Authors one grant of a new payment's approval workflow to one approver,
/// with the same facts the graph bootstrap binds for a pending payment.
pub(super) fn author_approval_grant(
    candidate: &mut Writer<'_>,
    payment: &BusinessPayment,
    business: &Entity<Business>,
    created: &Entity<PaymentIntent>,
    initiator: &Entity<Principal>,
    grantee: BankPrincipalId,
) -> Result<(), HandlerExecutionDenial> {
    let grantee_entity = candidate
        .resolve_entity(
            PrincipalIdentityField::at("PrincipalIdentityField"),
            grantee,
        )
        .map_err(HandlerExecutionDenial::new)?;
    let key_text = payment_workflow_grant_key(payment.id(), grantee);
    let key = WorthQueryApplicationEntityKey::<BankSchema, ApprovedBusinessPaymentGrant>::new(
        key_text.clone(),
    )
    .map_err(HandlerExecutionDenial::new)?;
    let grant = candidate
        .create_entity_in_context(business, ApprovedBusinessPaymentGrant::reference(), key)
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &grant,
            ApprovedBusinessPaymentGrantAction::at("ApprovedBusinessPaymentGrantAction"),
            "manage-approved-business-payment-workflow".to_owned(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &grant,
            ApprovedBusinessPaymentGrantPurpose::at("ApprovedBusinessPaymentGrantPurpose"),
            "business-payment-approval".to_owned(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &grant,
            ApprovedBusinessPaymentGrantStatus::at("ApprovedBusinessPaymentGrantStatus"),
            "active".to_owned(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &grant,
            ApprovedBusinessPaymentGrantWorkflow::at("ApprovedBusinessPaymentGrantWorkflow"),
            payment.id(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &grant,
            ApprovedBusinessPaymentGrantNotBefore::at("ApprovedBusinessPaymentGrantNotBefore"),
            0,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &grant,
            ApprovedBusinessPaymentGrantNotAfter::at("ApprovedBusinessPaymentGrantNotAfter"),
            u64::MAX,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &grant,
            ApprovedBusinessPaymentGrantDelegationLimit::at(
                "ApprovedBusinessPaymentGrantDelegationLimit",
            ),
            0,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            ApprovedBusinessPaymentGrantResource::reference(),
            format!("approved-payment-grant-resource:{key_text}"),
            &grant,
            created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            ApprovedBusinessPaymentGrantGrantor::reference(),
            format!("approved-payment-grant-grantor:{key_text}"),
            initiator,
            &grant,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            ApprovedBusinessPaymentGrantGrantee::reference(),
            format!("approved-payment-grant-grantee:{key_text}"),
            &grantee_entity,
            &grant,
        )
        .map_err(HandlerExecutionDenial::new)
}
