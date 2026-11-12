#![allow(
    clippy::disallowed_methods,
    reason = "These process courts own the Tokio test executor for real external-rail network I/O"
)]
//! Bank integration evidence grouped by its product frontier.
#![recursion_limit = "512"]

#[path = "worth_query_bank_court_external_rail/external_rail_tls/mod.rs"]
mod external_rail_tls;

#[path = "worth_query_bank_court_external_rail/process/mod.rs"]
mod process;
