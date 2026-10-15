//! The domain_contracts integration suite: one test binary for the crate's
//! scenario files, which stay in tests/ with their own module trees.
//! Support modules several scenarios share are compiled once here.

#[path = "ability_inventory.rs"]
mod ability_inventory;
#[path = "accounting_proposals.rs"]
mod accounting_proposals;
#[path = "create_personal_account_mutation_binding.rs"]
mod create_personal_account_mutation_binding;
#[path = "estate_aftermath_contract.rs"]
mod estate_aftermath_contract;
#[path = "estate_capability_installation.rs"]
mod estate_capability_installation;
#[path = "estate_disclosure_contract.rs"]
mod estate_disclosure_contract;
#[path = "estate_schema_inventory.rs"]
mod estate_schema_inventory;
#[path = "public_consumer_transcript.rs"]
mod public_consumer_transcript;
#[path = "schema_inventory.rs"]
mod schema_inventory;
#[path = "workflow_proposals.rs"]
mod workflow_proposals;
