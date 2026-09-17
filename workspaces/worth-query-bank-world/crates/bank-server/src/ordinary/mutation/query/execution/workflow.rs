use bank_domain::schema::{
    ApprovePayment, ApprovePaymentOperation, BankSchema, Business, InitiateBusinessPayment,
    InitiateBusinessPaymentOperation, PaymentIntent, RejectPayment, RejectPaymentOperation,
};
use worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions;

use crate::ordinary::mutation::{
    mutations, BankApprovePendingPayment, BankPaymentDecisionExecution,
    BankPaymentInitiationOutcome, BankRejectPendingPayment,
};
use crate::{BankIdentityRuntime, BankReadyMutation};

impl
    BankReadyMutation<
        '_,
        '_,
        mutations::InitiateBusinessPaymentMutation,
        InitiateBusinessPaymentOperation,
        Business,
    >
{
    pub fn execute(self) -> BankPaymentInitiationOutcome {
        execute_initiation(
            self.runtime,
            self.principal,
            self.preconditions,
            &self.controls,
            self.mutation.input,
        )
    }
}

impl BankReadyMutation<'_, '_, BankApprovePendingPayment, ApprovePaymentOperation, PaymentIntent> {
    pub fn execute(self) -> BankPaymentDecisionExecution {
        let input = ApprovePayment {
            payment: self.mutation.payment,
            approver: self.principal.principal_id(),
        };
        execute_approval(
            self.runtime,
            self.principal,
            self.preconditions,
            &self.controls,
            input,
        )
    }
}

impl BankReadyMutation<'_, '_, BankRejectPendingPayment, RejectPaymentOperation, PaymentIntent> {
    pub fn execute(self) -> BankPaymentDecisionExecution {
        let input = RejectPayment {
            payment: self.mutation.payment,
            rejecting_principal: self.principal.principal_id(),
        };
        execute_rejection(
            self.runtime,
            self.principal,
            self.preconditions,
            &self.controls,
            input,
        )
    }
}

impl
    BankReadyMutation<
        '_,
        '_,
        mutations::ApprovePaymentMutation,
        ApprovePaymentOperation,
        PaymentIntent,
    >
{
    pub fn execute(self) -> BankPaymentDecisionExecution {
        execute_approval(
            self.runtime,
            self.principal,
            self.preconditions,
            &self.controls,
            self.mutation.input,
        )
    }
}

impl
    BankReadyMutation<
        '_,
        '_,
        mutations::RejectPaymentMutation,
        RejectPaymentOperation,
        PaymentIntent,
    >
{
    pub fn execute(self) -> BankPaymentDecisionExecution {
        execute_rejection(
            self.runtime,
            self.principal,
            self.preconditions,
            &self.controls,
            self.mutation.input,
        )
    }
}

fn execute_approval(
    runtime: &BankIdentityRuntime,
    principal: &crate::BankAuthenticatedPrincipal,
    preconditions: TypedMutationPreconditions<BankSchema, ApprovePaymentOperation, PaymentIntent>,
    controls: &crate::BankMutationControls,
    input: ApprovePayment,
) -> BankPaymentDecisionExecution {
    runtime
        .request(principal, controls.request())
        .mutate(input)
        .preconditions(preconditions)
        .idempotency(controls.idempotency_key())
        .execute_in_program(runtime.application_program())
}

fn execute_rejection(
    runtime: &BankIdentityRuntime,
    principal: &crate::BankAuthenticatedPrincipal,
    preconditions: TypedMutationPreconditions<BankSchema, RejectPaymentOperation, PaymentIntent>,
    controls: &crate::BankMutationControls,
    input: RejectPayment,
) -> BankPaymentDecisionExecution {
    runtime
        .request(principal, controls.request())
        .mutate(input)
        .preconditions(preconditions)
        .idempotency(controls.idempotency_key())
        .execute_in_program(runtime.application_program())
}

fn execute_initiation(
    runtime: &BankIdentityRuntime,
    principal: &crate::BankAuthenticatedPrincipal,
    preconditions: TypedMutationPreconditions<
        BankSchema,
        InitiateBusinessPaymentOperation,
        Business,
    >,
    controls: &crate::BankMutationControls,
    input: InitiateBusinessPayment,
) -> BankPaymentInitiationOutcome {
    BankPaymentInitiationOutcome::from_program(
        runtime
            .request(principal, controls.request())
            .mutate(input)
            .preconditions(preconditions)
            .idempotency(controls.idempotency_key())
            .execute_in_program(runtime.application_program()),
    )
}
