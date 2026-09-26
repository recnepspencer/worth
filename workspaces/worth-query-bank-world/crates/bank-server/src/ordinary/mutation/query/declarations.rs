pub mod mutations {
    use bank_domain::schema::*;

    macro_rules! mutation {
        ($Name:ident, $Input:ty, $Operation:ty, $Scope:ty, $constructor:ident) => {
            #[derive(Clone, Debug, Eq, PartialEq)]
            pub struct $Name {
                pub(in crate::ordinary::mutation::query) input: $Input,
            }

            pub const fn $constructor(input: $Input) -> $Name {
                $Name { input }
            }

            impl super::BankMutationContract for $Name {
                type Operation = $Operation;
                type Scope = $Scope;
            }
        };
    }

    mutation!(
        CreatePersonalAccountMutation,
        CreatePersonalAccount,
        CreatePersonalAccountOperation,
        Institution,
        create_personal_account
    );
    mutation!(
        CreateBusinessAccountMutation,
        CreateBusinessAccount,
        CreateBusinessAccountOperation,
        Institution,
        create_business_account
    );
    mutation!(
        OpeningFundingMutation,
        ApplyOpeningFunding,
        ApplyOpeningFundingOperation,
        Institution,
        apply_opening_funding
    );
    mutation!(
        DepositMutation,
        Deposit,
        DepositOperation,
        Institution,
        deposit
    );
    mutation!(
        WithdrawalMutation,
        Withdraw,
        WithdrawOperation,
        Institution,
        withdraw
    );
    mutation!(
        SendMoneyMutation,
        SendMoney,
        SendMoneyOperation,
        Account,
        send_money
    );
    mutation!(
        InitiateBusinessPaymentMutation,
        InitiateBusinessPayment,
        InitiateBusinessPaymentOperation,
        Business,
        initiate_business_payment
    );
    mutation!(
        RejectPaymentMutation,
        RejectPayment,
        RejectPaymentOperation,
        PaymentIntent,
        reject_payment
    );
    mutation!(
        GrantAccountAccessMutation,
        GrantAccountAuthorization,
        GrantAccountAuthorizationOperation,
        Account,
        grant_account_access
    );
    mutation!(
        RevokeAccountAccessMutation,
        RevokeAccountAuthorization,
        RevokeAccountAuthorizationOperation,
        Account,
        revoke_account_access
    );
    mutation!(
        ReverseJournalMutation,
        ReverseJournal,
        ReverseJournalOperation,
        Institution,
        reverse_journal
    );
}

#[doc(hidden)]
pub trait BankMutationContract {
    type Operation;
    type Scope;
}

impl BankMutationContract for crate::BankRejectPendingPayment {
    type Operation = bank_domain::schema::RejectPaymentOperation;
    type Scope = bank_domain::schema::PaymentIntent;
}
