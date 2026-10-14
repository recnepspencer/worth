use bank_domain::schema::{
    Account, DepositOperation, Institution, SendMoneyOperation, WithdrawOperation,
};

use super::super::mutations;
use crate::ordinary::mutation::{BankMoneyMovementExecution, BankMoneyMovementRetainedExecution};
use crate::BankReadyMutation;

macro_rules! execute_program_mutation {
    ($Mutation:ty, $Operation:ty, $Scope:ty) => {
        impl BankReadyMutation<'_, '_, $Mutation, $Operation, $Scope> {
            pub fn execute(self) -> BankMoneyMovementExecution {
                self.runtime
                    .request(self.principal, self.controls.request())
                    .mutate(self.mutation.input)
                    .preconditions(self.preconditions)
                    .idempotency(self.controls.idempotency_key())
                    .execute_in_program(self.runtime.application_program(), worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
            }

            pub fn execute_retained(self) -> BankMoneyMovementRetainedExecution {
                self.runtime
                    .request(self.principal, self.controls.request())
                    .mutate(self.mutation.input)
                    .preconditions(self.preconditions)
                    .idempotency(self.controls.idempotency_key())
                    .execute_retained_in_program(self.runtime.application_program(), worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
            }
        }
    };
}

execute_program_mutation!(mutations::DepositMutation, DepositOperation, Institution);
execute_program_mutation!(
    mutations::WithdrawalMutation,
    WithdrawOperation,
    Institution
);
execute_program_mutation!(mutations::SendMoneyMutation, SendMoneyOperation, Account);
