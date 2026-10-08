use super::*;
use crate::{ExecutionAllocationDenialKind, ExecutionAllocationPolicy, ExecutionArrayBuilder};

#[test]
fn controlled_child_inherits_policy_and_exact_minimum_deadline_without_parent_mutation() {
    let _serial = TEST_LOCK.lock().unwrap();
    let parent_deadline = Instant::now() + Duration::from_secs(60);
    let mut parent_request = request(2, 8, 19);
    parent_request.deadline = Some(parent_deadline);
    let parent_token = parent_request.cancellation.clone();
    let parent = authority().request_lease(parent_request).unwrap();
    let earlier = parent_deadline - Duration::from_secs(1);
    let stop = CancellationToken::new();
    let child = parent.controlled_child(stop.clone(), Some(earlier));
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
        Err(LeaseDenial::ResourceExhausted)
    ));
    drop(array);
    drop(parent.reserve_memory(8).unwrap());

    let inherited = parent.controlled_child(CancellationToken::new(), None);
    parent_token.cancel();
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
