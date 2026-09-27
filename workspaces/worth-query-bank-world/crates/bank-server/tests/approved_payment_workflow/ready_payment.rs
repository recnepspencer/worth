//! One approved payment prepared to run against a live rail process.

use std::sync::Arc;
use std::time::Duration;

use bank_domain::schema::ApprovePayment;
use bank_external_rail::{test_control::FaultScript, RailProcessHandle};
use bank_server::{BankApprovedPaymentApplyOutcome, BankAuthenticatedPrincipal};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowOperation,
};

use super::assertions::key;
use super::authentication::approval_configuration;
use super::fixture::{
    ordinary_read_world_with_approval_authentication, principal_id, OrdinaryReadFixture, APPROVER,
};
use super::journey::prepare_approved_payment_operation;
use super::rail_transport::{spawn_rail, BankEstateRailTransport};
use super::support::request_scope;

pub(super) struct ReadyPaymentWorld {
    pub(super) fixture: OrdinaryReadFixture,
    pub(super) rail: Arc<BankEstateRailTransport>,
    pub(super) _rail_process: RailProcessHandle,
    pub(super) principal: BankAuthenticatedPrincipal,
    pub(super) scope: WorthQueryRequestScope,
    pub(super) authority: ApprovePayment,
    pub(super) instance: PublishedWorkflowInstanceRef,
    pub(super) operation: RequiredWorkflowOperation,
}

impl ReadyPaymentWorld {
    pub(super) fn new(scenario: &str, script: FaultScript) -> Self {
        let fixture =
            ordinary_read_world_with_approval_authentication(scenario, approval_configuration());
        let rail_process = spawn_rail();
        let rail = Arc::new(BankEstateRailTransport::connected_to(
            rail_process.local_addr(),
            rail_process.test_control_addr(),
        ));
        rail.under(script, Duration::from_millis(150));
        fixture
            .world
            .runtime
            .install_external_effect_transport(rail.clone())
            .expect("the real rail transport installs");
        let principal = fixture.authenticate(APPROVER);
        let scope = request_scope();
        let authority = ApprovePayment {
            payment: fixture.payment,
            approver: principal_id(APPROVER),
        };
        let workflow = fixture
            .world
            .runtime
            .approved_business_payment(&principal, &scope);
        let (instance, operation) = prepare_approved_payment_operation(&workflow, &authority);
        Self {
            fixture,
            rail,
            _rail_process: rail_process,
            principal,
            scope,
            authority,
            instance,
            operation,
        }
    }

    pub(super) fn perform(&self) -> BankApprovedPaymentApplyOutcome {
        self.fixture
            .world
            .runtime
            .approved_business_payment(&self.principal, &self.scope)
            .perform_apply(
                self.instance.clone(),
                &self.operation,
                self.authority.clone(),
                &key("approved-payment:operation:perform"),
            )
            .expect("the prepared Bank operation reaches Query's owner")
    }
}
