//! The applied round borrows its actual allocation owner and frozen mutation policy.
use crate::mvcc::publication::application_publication::BoundAuthoredCoupling;
use contracts::config::data::policies::MutationConfig;
use contracts::config::data::sections::StorageConfig;
use contracts::execution::request_work::RequestWorkBudget;
use contracts::symbols::data::client_key_symbol_policy::ClientKeySymbolPolicy;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::RecordIdentitySubsystem;

#[derive(Clone, Copy)]
pub struct AppliedMutationInputs<'a> {
    pub storage: &'a StorageConfig,
    pub key_policy: ClientKeySymbolPolicy,
    pub work: &'a Option<RequestWorkBudget>,
    pub record_identity: &'a RecordIdentitySubsystem,
    pub authored_coupling: Option<&'a std::sync::Arc<BoundAuthoredCoupling>>,
}

impl AppliedMutationInputs<'_> {
    pub(super) fn mutation_config(&self) -> MutationConfig {
        MutationConfig {
            cascade_delete_policy: self.storage.cascade_delete_policy,
            adjacency_policy: self.storage.adjacency_policy.clone(),
            cross_context_policy: self.storage.cross_context_policy,
        }
    }
}
