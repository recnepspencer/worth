use super::*;
use crate::{ExecutionAllocationDenialKind, ExecutionAllocationPolicy, ExecutionArrayBuilder};

#[test]
fn controlled_child_inherits_policy_and_exact_minimum_deadline_without_parent_mutation() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent_deadline = Instant::now() + Duration::from_secs(60);
    let mut parent_request = request(2, 8, 19);
    parent_request.deadline = Some(parent_deadline);
    let parent_source = CancellationSource::new();
    parent_request.cancellation = parent_source.token();
    let parent_token = parent_source.token();
    let parent = authority().request_lease(parent_request).unwrap();
    let earlier = parent_deadline - Duration::from_secs(1);
    let stop = CancellationSource::new();
    let child = parent.controlled_child(stop.token(), Some(earlier));
    assert_eq!(child.policy(), parent.policy());
    assert_eq!(child.deadline(), Some(earlier));
    assert_eq!(parent.deadline(), Some(parent_deadline));
    assert_eq!(
        parent
            .controlled_child(
                CancellationToken::new(),
                Some(parent_deadline + Duration::from_secs(1))
            )
            .deadline(),
        Some(parent_deadline)
    );
    assert_eq!(
        parent
            .controlled_child(CancellationToken::new(), None)
            .deadline(),
        Some(parent_deadline)
    );

    let mut builder =
        ExecutionArrayBuilder::<u64>::allocate(1, ExecutionAllocationPolicy::Execution(&child))
            .unwrap();
    builder.push(73).unwrap();
    let array = builder.seal().unwrap();
    stop.cancel();
    assert!(child.is_cancelled());
    assert_eq!(
        ExecutionAllocationPolicy::Execution(&child)
            .check_live()
            .unwrap_err()
            .kind(),
        ExecutionAllocationDenialKind::Cancelled,
    );
    assert!(!parent.is_cancelled());
    assert!(!parent_token.is_cancelled());
    drop(child);
    assert_eq!(array.elements(), &[73]);
    assert_eq!(array.charged_payload_bytes(), Some(8));
    assert!(matches!(
        parent.reserve_memory(1),
        Err(MemoryLimitDenial::Capacity {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 }
        })
    ));
    drop(array);
    drop(parent.reserve_memory(8).unwrap());

    let inherited = parent.controlled_child(CancellationToken::new(), None);
    parent_source.cancel();
    assert!(inherited.is_cancelled());
    let unrestricted = authority().request_lease(request(1, 0, 1)).unwrap();
    let expired = Instant::now() - Duration::from_secs(1);
    let elapsed = unrestricted.controlled_child(CancellationToken::new(), Some(expired));
    assert_eq!(elapsed.deadline(), Some(expired));
    assert!(elapsed.deadline_elapsed());
    assert_eq!(
        ExecutionAllocationPolicy::Execution(&elapsed)
            .check_live()
            .unwrap_err()
            .kind(),
        ExecutionAllocationDenialKind::DeadlineElapsed,
    );
    assert!(!unrestricted.deadline_elapsed());
}

#[test]
fn controlled_child_made_inside_a_descendant_worker_descends_from_that_worker() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent = authority().request_lease(request(1, 4096, 100)).unwrap();
    let call = parent.controlled_child(CancellationToken::new(), None);
    let (charged, _) = crate::ExecutionRequest::leased(&call)
        .run(crate::ExecutionWorkCeiling::new(None), |_| {
            // An attempt made from the caller's lease while the call's worker
            // runs: beside that worker it would refuse as an unrelated lease.
            let attempt = parent.controlled_child(CancellationToken::new(), None);
            assert_eq!(attempt.lineage_depth(), call.lineage_depth() + 1);
            assert_eq!(attempt.policy(), parent.policy());
            ExecutionAllocationPolicy::Execution(&attempt)
                .check_live()
                .unwrap();
            let before = parent.charged_memory_bytes();
            let mut builder = ExecutionArrayBuilder::<u64>::allocate(
                1,
                ExecutionAllocationPolicy::Execution(&attempt),
            )
            .unwrap();
            builder.push(5).unwrap();
            let array = builder.seal().unwrap();
            assert_eq!(array.elements(), &[5]);
            parent.charged_memory_bytes() - before
        })
        .unwrap();
    assert_eq!(charged, 8, "the attempt's charge reaches the caller's lease");
    // Outside any worker the child still descends from the lease itself.
    let outside = parent.controlled_child(CancellationToken::new(), None);
    assert_eq!(outside.lineage_depth(), parent.lineage_depth() + 1);
}
