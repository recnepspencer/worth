mod coordination;
mod discovery;
mod handoff;
mod manifest_facts;
mod planning;
mod publication;
mod reader_limit;
mod recovery;
mod recovery_budget;
/// Tests outside orchestration mint their expected limits through its door.
#[cfg(test)]
pub(crate) use recovery_budget::recovery_limit_for_test;
mod reopen;
mod staging;
mod wal_residency;
pub(crate) mod wal_selection;

pub(crate) use coordination::RecoveryCoordination;
pub(crate) use discovery::source_memory::source_memory_limit;
pub(crate) use discovery::{
    discover_sources, AdmittedWalInventory, BootstrapDiscovery, CheckpointDiscovery,
    DiscoveryMaterial, WalDiscovery,
};
pub(crate) use handoff::{finish_recovery_after_cleanup, RecoveryHandoffBacking};
pub(crate) use manifest_facts::{ManifestFactsDiscovery, ManifestFactsState};
pub(crate) use planning::{
    plan_recovery, ExceededManifestEntries, ValidatedManifestResidueCleanup,
};
pub(crate) use publication::{publish_recovery, PublicationPayload, PublicationStateBacking};
pub(crate) use recovery::recover;
pub(crate) use reopen::{prepare_selected_custody_rebind, reopen_recovery};
pub(crate) use staging::{
    stage_recovery, RecoveryStagingCancellation, RecoveryStagingInput, RecoveryStagingInputBacking,
    RecoveryStagingInputState,
};
pub(crate) use wal_residency::NativeWalRoster;
pub(crate) use wal_selection::ResidentSourceSelection;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
pub(crate) mod source_copy;
