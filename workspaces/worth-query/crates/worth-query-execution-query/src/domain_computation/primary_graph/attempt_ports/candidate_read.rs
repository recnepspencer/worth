//! `CandidateReadSupportPort` implementations for applied query and verified parent reads.

use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    mvcc::RequiredOutputPublicationMeaning,
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
};
use worth_query_execution_attempt::facade::ports::CandidateReadSupportPort;

use crate::domain_computation::primary_graph::{
    application_attempt::{
        WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact as Fact,
    },
    application_contribution::producer::demand::selection::native_selection::current_verification::applied::VerifiedAppliedParent,
    application_query::observed_source::applied_support::AppliedQuerySupport,
    provider::ParentRead,
};

impl CandidateReadSupportPort for AppliedQuerySupport {
    fn subject(&self) -> &str {
        self.query()
    }

    fn pin(&self) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        self.pin()
    }

    fn changed_fact<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        earlier: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        image: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<Option<&'a Fact>, WorthQueryApplicationAttemptDenial> {
        self.changed_fact(execution, runtime, before, earlier, image)
    }
}

impl CandidateReadSupportPort for VerifiedAppliedParent {
    fn subject(&self) -> &str {
        self.family()
    }

    fn pin(&self) -> &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity> {
        self.pin()
    }

    fn changed_fact<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        before: &SnapshotHandle,
        earlier: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        image: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<Option<&'a Fact>, WorthQueryApplicationAttemptDenial> {
        self.changed_fact(execution, runtime, before, earlier, image)
    }

    fn has_route(&self, meaning: &RequiredOutputPublicationMeaning) -> bool {
        self.has_route(meaning)
    }

    fn read(&self) -> Option<ParentRead> {
        self.read()
    }
}
