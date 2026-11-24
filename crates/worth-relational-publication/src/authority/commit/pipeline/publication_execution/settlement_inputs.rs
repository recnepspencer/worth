//! Install only the prepared effects authorized by a performed movement.
use worth_relational_branch::internal_port::branch::head_visibility_inputs::BranchHeadVisibilityInputs;
use worth_relational_contracts::internal_port::config::data::policies::DurableLogPolicy;
use crate::publication::authority::PublicationAuthority;
use worth_relational_durability::internal_port::runtime::state::subsystems::durability::commit_log::DurabilityCommitLog;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::LineageSubsystem;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use worth_relational_storage::internal_port::storage::changed_record_retention::ChangedRecordRetention;

pub struct PublicationSettlementInputs<'a> {
    pub history: &'a HistorySubsystem,
    pub lineage: &'a LineageSubsystem,
    pub visibility: BranchHeadVisibilityInputs<'a>,
    pub retention: ChangedRecordRetention<'a>,
    pub log: DurabilityCommitLog<'a>,
    pub log_policy: &'a DurableLogPolicy,
    pub publication: PublicationAuthority<'a>,
    pub instrumentation: &'a RuntimeInstrumentation,
}
