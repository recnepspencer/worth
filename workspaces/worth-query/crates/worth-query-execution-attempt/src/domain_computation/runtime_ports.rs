//! Newtype wrapper for `Arc<WorthQueryPrimaryGraphProvider>` used by lower runtime traits.
use std::ops::Deref;
use std::sync::Arc;

use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;

#[derive(Clone)]
pub struct PrimaryGraphProviderArc(pub Arc<WorthQueryPrimaryGraphProvider>);

impl Deref for PrimaryGraphProviderArc {
    type Target = WorthQueryPrimaryGraphProvider;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use std::any::Any;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableRecoveryClaimIdentity, RecoveryClaimDisposition,
};
use worth_query_execution_contracts::facade::{
    QueryDispatchJournalDenial, QueryRecordedRecoveryClaim,
};
use worth_query_execution_runtime::facade::ports::{
    PrimaryReadCustody, RecoveryDispatchJournalPort,
};
use crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal;
use crate::domain_computation::primary_graph::provider::RegisteredPrimaryRead;

impl PrimaryReadCustody for RegisteredPrimaryRead {
    fn grouped_count(&self) -> usize {
        self.facts().grouped_count()
    }

    fn contains_key(&self, identity: &str) -> bool {
        self.facts().contains_key(identity)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

impl RecoveryDispatchJournalPort for WorthQueryDispatchJournal {
    fn dispose_recovery_claim(
        &self,
        original: &DurableCompositeCommitIdentity,
        identity: DurableRecoveryClaimIdentity,
        terminal: RecoveryClaimDisposition,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        self.begin_recovery_admission(request)?
            .dispose(original, identity, terminal, request)
    }

    fn recorded_recovery_claim(
        &self,
        original: &DurableCompositeCommitIdentity,
    ) -> Result<Option<QueryRecordedRecoveryClaim>, QueryDispatchJournalDenial> {
        WorthQueryDispatchJournal::recorded_recovery_claim(self, original)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

use worth_query_execution_runtime::facade::ports::BranchCommitCoordinationPort;
use worth_runtime_world::facade::ProductBranchIncarnation;

impl BranchCommitCoordinationPort for WorthQueryPrimaryGraphProvider {
    fn with_occurrence_coordination(
        &self,
        occurrence: ProductBranchIncarnation,
        body: &mut dyn FnMut(),
    ) {
        let lane = self.application_branch_commit_lane_for_occurrence(occurrence);
        let _guard = lane.enter();
        body();
    }
}

impl PartialEq for PrimaryGraphProviderArc {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for PrimaryGraphProviderArc {}
