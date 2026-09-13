use std::sync::Arc;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::history::CompositeRuntimeWorldCommit;
use crate::publication::CompositeOwnerExecutionResults;

use super::ActiveAttemptCustody;

impl ActiveAttemptCustody {
    pub(crate) fn begin_owner_execution(&mut self) {
        self.lease_resources()
            .resources_mut()
            .operation
            .as_mut()
            .expect("active custody holds operation admission")
            .begin_owner_execution()
            .expect("a reserved attempt begins owner execution");
    }

    pub(crate) fn begin_publication(&mut self) {
        self.lease_resources()
            .resources_mut()
            .operation
            .as_mut()
            .expect("active custody holds operation admission")
            .begin_publication()
            .expect("an executing attempt enters publication");
    }

    pub(crate) fn begin_recovery(&mut self) {
        self.lease_resources()
            .resources_mut()
            .operation
            .as_mut()
            .expect("active custody holds operation admission")
            .begin_recovery()
            .expect("a publishing attempt enters recovery");
    }

    /// The immutable occurrence is stored before any pin acquisition. A
    /// binding panic leaves the original record and reservation reachable.
    pub(crate) fn prepare_commit(
        &mut self,
        successor: AdmittedCompositeRuntimeWorldBasis,
        results: &CompositeOwnerExecutionResults,
    ) -> Arc<CompositeRuntimeWorldCommit> {
        let expected = self.record.expected.clone();
        let attempt = self.record.attempt.clone();
        let mut lease = self.lease_resources();
        let resources = lease.resources_mut();
        if let Some(commit) = &resources.commit {
            assert_eq!(commit.basis(), &successor);
            assert!(commit.matches_owner_results(expected.basis(), results));
            return Arc::clone(commit);
        }
        let commit = Arc::new(
            CompositeRuntimeWorldCommit::from_ordinary_publication(
                resources.commit_identity.clone(),
                expected.snapshot().commit(),
                successor,
                attempt,
                results,
                None,
            )
            .expect("owner-issued progress and the admitted successor form one commit"),
        );
        resources.commit = Some(Arc::clone(&commit));
        drop(lease);
        self.record_successor(commit.basis().clone());
        commit
    }
}
