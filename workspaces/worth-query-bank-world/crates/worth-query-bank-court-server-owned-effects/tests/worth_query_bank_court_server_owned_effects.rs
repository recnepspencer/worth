//! Bank integration evidence grouped by its required dependency surface.
#![recursion_limit = "512"]

#[path = "worth_query_bank_court_server_owned_effects/oracle/mod.rs"]
mod oracle;

#[path = "worth_query_bank_court_server_owned_effects/process/mod.rs"]
mod process;
