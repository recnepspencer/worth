//! Accepted immutable pin custody outlives a later cold transport refusal.
use super::*;
use crate::runtime::RelationalBorrowedRecordReadDenial;
use crate::visibility::materialization::read_records::ProjectionAspectScope;
#[test]
fn accepted_pin_projections_survive_later_proven_member_damage() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for (cause, sticky) in FAILURES {
        let fixture = CurrentFixture::reopened(execution);
        let snapshot = snapshot_for_owner_branch(
            &fixture.runtime,
            &crate::history::data::BranchId("main".into()),
        );
        let pin = fixture
            .runtime
            .read_truth()
            .pin_read_capability(&snapshot, &[], execution)
            .unwrap();
        let accepted = pin.project_snapshot(&snapshot).unwrap();
        let fresh = fixture
            .runtime
            .read_truth()
            .project_snapshot(&snapshot)
            .unwrap();
        let target = fixture.source.member_reads.lock().unwrap()[0];
        assert!(
            fixture.heads.contains(&target),
            "pin first loads its actual selected Head"
        );
        let frontier = fixture.runtime.history().latest_patch_stream_position();
        let offered = attached_test_medium(&fixture.runtime).offered().len();
        fixture
            .source
            .failures
            .lock()
            .unwrap()
            .insert(ColdRead::Member(target), cause);
        let refusal = fixture
            .runtime
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .unwrap_err();
        let crate::durability::data::RelationalNativeCheckpointCaptureDenial::Durability(error) =
            refusal
        else {
            panic!("actual native export refuses its selected cold frame")
        };
        assert_eq!(error.storage_unavailable.unwrap().cause, cause);
        fixture.source.failures.lock().unwrap().clear();
        let reads = fixture.source.reads.load(Ordering::Relaxed);
        if sticky {
            assert!(
                matches!(fresh.read_exact_entity_state(execution, fixture.entity, |metadata, _| metadata.entity_id), Err(RelationalBorrowedRecordReadDenial::Execution(crate::execution::RelationalExecutionDenialCause::StorageUnavailable(failure))) if failure.cause == cause)
            );
        } else {
            assert_eq!(
                fresh
                    .read_exact_entity_state(execution, fixture.entity, |metadata, _| metadata
                        .entity_id)
                    .unwrap(),
                Some(fixture.entity)
            );
        }
        assert_eq!(
            accepted
                .read_exact_entity_state(execution, fixture.entity, |metadata, _| metadata
                    .entity_id)
                .unwrap(),
            Some(fixture.entity),
            "accepted pin x {cause:?}"
        );
        assert_eq!(
            accepted
                .entity_record_with_projection_scope(
                    execution,
                    fixture.entity,
                    ProjectionAspectScope::empty(),
                    |_| Some(true)
                )
                .unwrap(),
            Some(true)
        );
        assert!(!accepted
            .bounded_entities_of_kind(execution, KindId(1), usize::MAX)
            .unwrap()
            .unwrap()
            .records()
            .is_empty());
        assert_eq!(
            fixture.source.reads.load(Ordering::Relaxed),
            reads,
            "accepted pins never reacquire cold meaning"
        );
        assert_eq!(
            fixture.runtime.history().latest_patch_stream_position(),
            frontier
        );
        assert_eq!(
            attached_test_medium(&fixture.runtime).offered().len(),
            offered
        );
        fixture
            .runtime
            .snapshots()
            .release_snapshot(&snapshot)
            .unwrap();
    }
}
