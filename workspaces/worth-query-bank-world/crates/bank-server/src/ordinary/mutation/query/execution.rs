#![allow(
    clippy::result_large_err,
    reason = "Owning mutation construction must return the original input key and cleanup custody on refusal"
)]

mod account_access;
mod account_creation;
mod deposit;
mod send_money;
mod withdraw;
mod workflow;

mod estate_administration;
mod estate_settlement;
mod opening_funding;
mod reverse_journal;
