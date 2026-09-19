use worth_query_decl::facade::worth_query_application;

use super::contributions::{BankAccounts, BankEstate, BankPayments};

worth_query_application! {
    pub BankSchema {
        owner: "WORTH.bank",
        version: (1, 0),
        contributions: [BankAccounts, BankPayments, BankEstate],
    }
}
