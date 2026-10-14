//! Exact installed money-action occurrences and retained commit delivery.
use super::composition::BankPaymentsFeature;
use bank_domain::schema::*;
use worth_query_host::facade::declaration::application_program::{
    ApplicationActionDelivery, ApplicationActionReference, ApplicationRetainedCommit,
    ApplicationRootComposition,
};

macro_rules! retained_money_action {
    ($action:ident, $delivery:ident, $binding:ty, $identity:literal) => {
        pub struct $delivery;
        impl ApplicationActionDelivery<BankSchema, $binding> for $delivery {
            type Mode = ApplicationRetainedCommit;
            const IDENTITY: &'static str = $identity;
        }
        pub type $action = ApplicationActionReference<
            BankSchema,
            ApplicationRootComposition,
            BankPaymentsFeature,
            $binding,
            $delivery,
        >;
    };
}
retained_money_action!(
    OpeningFundingAction,
    OpeningFundingDelivery,
    ApplyOpeningFundingMutationBinding,
    "bank.money.opening-funding.retained.v1"
);
retained_money_action!(
    DepositAction,
    DepositDelivery,
    DepositMutationBinding,
    "bank.money.deposit.retained.v1"
);
retained_money_action!(
    WithdrawalAction,
    WithdrawalDelivery,
    WithdrawMutationBinding,
    "bank.money.withdrawal.retained.v1"
);
retained_money_action!(
    SendMoneyAction,
    SendMoneyDelivery,
    SendMoneyMutationBinding,
    "bank.money.send.retained.v1"
);
retained_money_action!(
    ReverseJournalAction,
    ReverseJournalDelivery,
    ReverseJournalMutationBinding,
    "bank.money.reverse.retained.v1"
);
