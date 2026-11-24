#[cfg(test)]
mod tests {
    use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeFixturePort as _;

    use super::super::position_admission::CanonicalPositionAdmission;
    use worth_relational_branch::internal_port::runtime::state::subsystems::history::canonical_publication_routes::retained_position::RetainedCanonicalPosition;
    use crate::durability::canonical_commit_artifact::checkpoint_body::copy_original_for_test;
    use crate::history::data::PositionedCanonicalCommit;
    use crate::tests::support::*;
    use std::sync::Arc;
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
        let copy = Arc::new(copy_original_for_test(sealed.envelope(), execution));
        let admitted = CanonicalPositionAdmission::equivalent(&sealed, copy.clone()).unwrap();
        let rebound = PositionedCanonicalCommit::admit(admitted);
        assert_eq!(rebound.position(), sealed.position());
        assert!(Arc::ptr_eq(rebound.canonical_arc(), &copy));
        let mut different = copy_original_for_test(&copy, execution);
        different.body_mut_for_test().commit.commit_id.0 += 1;
        assert!(CanonicalPositionAdmission::equivalent(&sealed, Arc::new(different)).is_err());
    }
}
