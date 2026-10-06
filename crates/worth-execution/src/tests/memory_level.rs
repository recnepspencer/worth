//! A memory refusal names the limit that refused. A request over its own
//! policy reads the same whatever other requests hold; only the process's
//! refusal is one that another request's release can clear.

use super::*;

#[test]
fn a_refusal_names_the_process_or_the_policy_that_refused() {
    let _serial = TEST_LOCK.lock().unwrap();
    let authority = authority();
    let small = authority.request_lease(request(1, 300, 10)).unwrap();
    let over_its_own = MemoryLimitDenial {
        requested: 400,
        admitted: 300,
        level: MemoryLimitLevel::Policy { ancestor: 0 },
    };
    assert_eq!(small.reserve_memory(400).unwrap_err(), over_its_own);

    let other = authority.request_lease(request(1, 2_000, 10)).unwrap();
    let held = other.reserve_memory(1_700).unwrap();
    assert_eq!(
        small.reserve_memory(400).unwrap_err(),
        over_its_own,
        "another request's hold leaves a policy refusal as it was"
    );
    let roomy = authority.request_lease(request(1, 1_000, 10)).unwrap();
    assert_eq!(
        roomy.reserve_memory(400).unwrap_err(),
        MemoryLimitDenial {
            requested: 400,
            admitted: 300,
            level: MemoryLimitLevel::Process,
        },
        "the same numbers, refused by the process another request fills"
    );
    drop(held);
    let on_roomy = roomy.reserve_memory(800).unwrap();

    let nested = roomy.child(request(1, 1_000, 10)).unwrap();
    assert_eq!(
        nested.reserve_memory(400).unwrap_err(),
        MemoryLimitDenial {
            requested: 400,
            admitted: 200,
            level: MemoryLimitLevel::Policy { ancestor: 1 },
        },
        "a parent's policy is named by how far out it is"
    );
    drop(on_roomy);
    assert_eq!(nested.reserve_memory(400).unwrap().bytes(), 400);
}
