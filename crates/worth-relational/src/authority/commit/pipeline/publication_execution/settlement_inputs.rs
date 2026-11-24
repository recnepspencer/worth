//! Install only the prepared effects authorized by a performed movement.
use crate::branch::head_visibility_inputs::BranchHeadVisibilityInputs;
use crate::config::data::DurableLogPolicy;
use crate::publication::authority::PublicationAuthority;
use crate::runtime::state::subsystems::durability::commit_log::DurabilityCommitLog;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::runtime::state::subsystems::lineage::LineageSubsystem;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::storage::changed_record_retention::ChangedRecordRetention;

pub(crate) struct PublicationSettlementInputs<'a> {
    pub(crate) history: &'a HistorySubsystem,
    pub(crate) lineage: &'a LineageSubsystem,
    pub(crate) visibility: BranchHeadVisibilityInputs<'a>,
    pub(crate) retention: ChangedRecordRetention<'a>,
    pub(crate) log: DurabilityCommitLog<'a>,
    pub(crate) log_policy: &'a DurableLogPolicy,
    pub(crate) publication: PublicationAuthority<'a>,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
}
