//! Retain admitted position provenance without retaining its envelope payload.
use super::position_admission::CanonicalPositionAdmission;
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::publication::patch::data::PatchStreamPosition;
use std::sync::{Arc, Weak};

#[derive(Debug, Clone)]
pub(crate) struct RetainedCanonicalPosition {
    position: PatchStreamPosition,
    envelope: Weak<CanonicalCommitEnvelope>,
}
impl RetainedCanonicalPosition {
    /// The sealed input binds this exact body allocation to its position.
    pub(crate) fn from_positioned(commit: &PositionedCanonicalCommit) -> Self {
        Self {
            position: commit.position(),
            envelope: Arc::downgrade(commit.canonical_arc()),
        }
    }
    pub(crate) fn position(&self) -> PatchStreamPosition {
        self.position
    }
    pub(crate) fn resident(&self) -> Option<PositionedCanonicalCommit> {
        CanonicalPositionAdmission::retained(self).map(PositionedCanonicalCommit::admit)
    }
    pub(super) fn upgrade_parts(
        &self,
    ) -> Option<(PatchStreamPosition, Arc<CanonicalCommitEnvelope>)> {
        self.envelope
            .upgrade()
            .map(|envelope| (self.position, envelope))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::*;
    #[test]
    fn position_retention_keeps_provenance_while_only_a_real_reader_keeps_the_body() {
        let host = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host);
        let sealed = {
            let runtime = runtime_with_test_schema();
            create_entity(execution, &runtime, "position-retention");
            runtime
                .history
                .positioned_canonical_commits_snapshot()
                .pop()
                .unwrap()
        };
        let position = sealed.position();
        let retained = RetainedCanonicalPosition::from_positioned(&sealed);
        let reader = retained.resident().unwrap();
        assert_eq!(reader.position(), position);
        assert!(Arc::ptr_eq(reader.canonical_arc(), sealed.canonical_arc()));
        drop(sealed);
        assert!(retained.resident().is_some());
        drop(reader);
        assert!(
            retained.resident().is_none(),
            "the witness owns no envelope payload"
        );
        assert_eq!(retained.position(), position);
    }
    #[test]
    fn equivalent_position_seals_require_the_same_canonical_payload() {
        let host = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host);
        let sealed = {
            let runtime = runtime_with_test_schema();
            create_entity(execution, &runtime, "equivalent-position");
            runtime
                .history
                .positioned_canonical_commits_snapshot()
                .pop()
                .unwrap()
        };
        let copy = Arc::new(sealed.envelope().clone());
        let admitted = CanonicalPositionAdmission::equivalent(&sealed, copy.clone()).unwrap();
        let rebound = PositionedCanonicalCommit::admit(admitted);
        assert_eq!(rebound.position(), sealed.position());
        assert!(Arc::ptr_eq(rebound.canonical_arc(), &copy));
        let mut different = copy.as_ref().clone();
        different.commit.commit_id.0 += 1;
        assert!(CanonicalPositionAdmission::equivalent(&sealed, Arc::new(different)).is_err());
    }
}
