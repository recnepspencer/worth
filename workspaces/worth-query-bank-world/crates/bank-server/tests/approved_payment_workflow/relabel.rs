use super::*;
use bank_domain::proposals::BankProposalDenial;
use bank_server::BankApprovedPaymentApplyOutcome;
use fixture::{ordinary_read_world_with_two_approvers, STRANGER};

#[test]
fn authenticated_actor_cannot_be_relabelled_in_payment_input() {
    let fixture = ordinary_read_world_with_two_approvers(
        "approved-payment-relabel",
        authentication::approval_configuration(),
    );
    let rail_process = spawn_rail();
    let settlement_rail = Arc::new(BankEstateRailTransport::connected_to(
        rail_process.local_addr(),
        rail_process.test_control_addr(),
    ));
    fixture
        .world
        .runtime
        .install_external_effect_transport(settlement_rail.clone())
        .expect("the Bank rail owner installs");
    let actor = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let workflow = fixture
        .world
        .runtime
        .approved_business_payment(&actor, &scope);
    // The actor holds approval authority, but names another approver in the
    // payment input; the workflow carries that input to the payment operation,
    // whose handler compares it with the authenticated principal.
    let relabelled = ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(STRANGER),
    };
    let (instance, operation) = journey::prepare_approved_payment_operation(&workflow, &relabelled);
    let denied = workflow
        .perform_apply(
            instance,
            &operation,
            relabelled,
            &key("approved-payment:relabel:perform"),
        )
        .expect("the relabelled operation reaches the Bank payment handler");
    assert!(
        matches!(
            denied,
            BankApprovedPaymentApplyOutcome::DomainDenied(
                BankProposalDenial::AuthenticatedActorMismatch
            )
        ),
        "the payment handler must refuse a relabelled approver: {denied:?}"
    );
    assert!(settlement_rail.attempts().is_empty());
}
