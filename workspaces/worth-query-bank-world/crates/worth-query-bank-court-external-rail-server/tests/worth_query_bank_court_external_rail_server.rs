//! Bank integration evidence grouped by its product frontier.
#![recursion_limit = "512"]

#[path = "worth_query_bank_court_external_rail_server/server/mod.rs"]
mod server;

#[path = "worth_query_bank_court_external_rail_server/cost/mod.rs"]
mod cost;

#[path = "worth_query_bank_court_external_rail_server/oracle/mod.rs"]
mod oracle;

#[path = "worth_query_bank_court_external_rail_server/process/mod.rs"]
mod process;
