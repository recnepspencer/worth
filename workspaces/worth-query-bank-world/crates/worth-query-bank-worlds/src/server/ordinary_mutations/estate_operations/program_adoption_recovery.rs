//! Performed effect recovery across branch-local program adoption.

pub use bank_domain::estate::EstateAction;
pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::LedgerStatus;
pub use bank_server::BankApplicationP1;
pub use worth_query_host::facade::application_entry::WorthQueryBranchAdoptionPublicationOutcome;
pub use worth_query_host::facade::declaration::application_program::ApplicationSemanticChangeKind;
pub use worth_query_host::facade::domain::WorthQueryProgramCustodyInventoryKind;
pub use worth_query_host::facade::primary_graph::{
    WorthQueryProgramCustodyDispositionInventory, WorthQueryProgramCustodyDispositionKind,
};

pub use super::phase8_cross_gate::world;
pub use crate::facade::server::support::request_scope;

pub fn has_exact_recovery(
    inventory: &WorthQueryProgramCustodyDispositionInventory,
    requirement: &worth_query_host::facade::domain::WorthQueryProgramCustodyInventoryRequirement,
) -> bool {
    inventory.dispositions().iter().any(|disposition| {
        disposition.requirement() == requirement
            && disposition.kind()
                == WorthQueryProgramCustodyDispositionKind::RetainExactEffectRecovery
    })
}
