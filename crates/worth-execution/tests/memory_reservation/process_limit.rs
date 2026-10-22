//! A finite process policy is exercised in its actual owning executable,
//! independently of work, worker, and canonical-settlement fixtures.
use super::*;

#[test]
fn a_refusal_names_the_process_or_the_policy_that_refused() {
    let _serial = test_serialization::guard(&SERIAL);
    let authority = authority();
    assert_eq!(
        authority
            .request_lease(request(PROCESS_BYTES + 1))
            .unwrap_err(),
        LeaseDenial::MemoryLimitExceedsParent
    );
    let small = authority.request_lease(request(300)).unwrap();
    let over_its_own = MemoryLimitDenial::Capacity {
        requested: 400,
        admitted: 300,
        level: MemoryLimitLevel::Policy { ancestor: 0 },
    };
    assert_eq!(small.reserve_memory(400).unwrap_err(), over_its_own);

    let other = authority.request_lease(request(PROCESS_BYTES)).unwrap();
    let held = other.reserve_memory(PROCESS_BYTES - 300).unwrap();
    assert_eq!(
        small.reserve_memory(400).unwrap_err(),
        over_its_own,
        "another request's hold leaves a policy refusal unchanged"
    );
    let roomy = authority.request_lease(request(1_000)).unwrap();
    assert_eq!(
        roomy.reserve_memory(400).unwrap_err(),
        MemoryLimitDenial::Capacity {
            requested: 400,
            admitted: 300,
            level: MemoryLimitLevel::Process,
        },
        "another request's live allocation exhausts the actual process policy"
    );
    drop(held);
    let on_roomy = roomy.reserve_memory(800).unwrap();
    let nested = roomy.child(request(1_000)).unwrap();
    assert_eq!(
        nested.reserve_memory(400).err(),
        Some(MemoryLimitDenial::Capacity {
            requested: 400,
            admitted: 200,
            level: MemoryLimitLevel::Policy { ancestor: 1 },
        }),
        "nested memory respects its original parent's remaining policy room"
    );
    drop(on_roomy);
    assert_eq!(nested.reserve_memory(400).unwrap().bytes(), 400);
    assert_eq!(roomy.charged_memory_bytes(), 0);
}
