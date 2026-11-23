//! The applied round borrows its actual allocation owner and frozen mutation policy.
use crate::config::data::{MutationConfig, StorageConfig};
use crate::execution::RequestWorkBudget;
use crate::mvcc::publication::application_publication::BoundAuthoredCoupling;
use crate::runtime::state::subsystems::record_identity::RecordIdentitySubsystem;
use crate::symbols::data::ClientKeySymbolPolicy;

#[derive(Clone, Copy)]
pub(crate) struct AppliedMutationInputs<'a> {
    pub(crate) storage: &'a StorageConfig,
    pub(crate) key_policy: ClientKeySymbolPolicy,
    pub(crate) work: &'a Option<RequestWorkBudget>,
    pub(crate) record_identity: &'a RecordIdentitySubsystem,
    pub(crate) authored_coupling: Option<&'a std::sync::Arc<BoundAuthoredCoupling>>,
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
