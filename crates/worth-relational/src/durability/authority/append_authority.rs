use crate::history::data::CanonicalCommitEnvelope;
use crate::history::data::{BranchId, CommitId};

/// Runtime-bound permission to append one exact canonical commit envelope.
///
/// The authority is intentionally non-cloneable. Its only constructors consume
/// sealed admissions minted by the typed commit publication lane.
pub(crate) struct DurableAppendAuthority {
    runtime_instance_id: u64,
    commit_id: CommitId,
    branch_id: BranchId,
}

impl DurableAppendAuthority {
    pub(crate) fn from_commit(
        admission: crate::authority::commit::CommitDurableAppendAdmission,
    ) -> Self {
        let (runtime_instance_id, commit_id, branch_id) = admission.into_parts();
        Self {
            runtime_instance_id,
            commit_id,
            branch_id,
        }
    }

    pub(crate) fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    pub(crate) fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub(super) fn validate(
        &self,
        runtime_instance_id: u64,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<(), crate::durability::data::DurabilityError> {
        let exact_runtime = self.runtime_instance_id == runtime_instance_id;
        let exact_commit = self.commit_id == envelope.commit.commit_id;
        let exact_branch = self.branch_id == envelope.commit.branch_id;
        if exact_runtime && exact_commit && exact_branch {
            return Ok(());
        }
        Err(crate::durability::data::DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::DurableIoFailure,
            "durable append authority does not match the runtime and canonical commit envelope",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::DurabilityRead;
    use crate::tests::support::{create_entity_outcome, runtime_with_test_schema};

    #[test]
    fn foreign_runtime_authority_cannot_append_an_exact_commit_envelope() {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let source = runtime_with_test_schema();
        let committed = create_entity_outcome(execution, &source, "durable-authority-source");
        let envelope = committed.envelope().clone();
        let positioned = crate::history::data::PositionedCanonicalCommit::for_test(
            committed.patch_position(),
            std::sync::Arc::new(envelope.clone()),
        );
        let authority = DurableAppendAuthority {
            runtime_instance_id: source.runtime_instance_id(),
            commit_id: envelope.commit.commit_id,
            branch_id: envelope.commit.branch_id.clone(),
        };
        let foreign = runtime_with_test_schema();
        let durable_count_before = foreign.durable_log().len();

        let error = foreign
            .durability_authority()
            .append_commit(authority, &positioned)
            .expect_err("runtime-affine durable append authority must reject a foreign runtime");

        assert_eq!(
            error.class,
            crate::durability::data::RecoveryFailureClass::DurableIoFailure
        );
        assert_eq!(foreign.durable_log().len(), durable_count_before);
    }
}
