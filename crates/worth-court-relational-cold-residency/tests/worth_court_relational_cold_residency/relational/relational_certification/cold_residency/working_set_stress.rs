//! S1 stress: eight concurrent public scans of a Home reopened under a tight
//! `ResidencyBudget` never deadlock, stay within the working set, and each
//! reads the whole root through eviction and basis-checked rewarm.
use super::working_set_fixture::*;
use worth_execution::ExecutionRequest;

#[test]
fn eight_concurrent_scans_never_deadlock_and_each_read_the_whole_root() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let home = cold_home(request);
    let head = head_region_bytes(request, &home);
    // Eight one-region windows fit, the whole root does not.
    let ceiling = 10 * head;
    let runtime = reopen(request, &home, budget(ceiling, ceiling));
    let start = std::sync::Barrier::new(8);
    std::thread::scope(|scope| {
        let workers = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
                    let request = &ExecutionRequest::serial(&host);
                    start.wait();
                    let (mut completed, mut refused) = (0, 0);
                    while completed < 3 {
                        let (_, basis) = runtime
                            .observe_branch(&runtime.main_branch_identity())
                            .unwrap();
                        match scan_observation(request, &runtime, &basis.observation()) {
                            Ok(read) => {
                                assert_eq!(read, REGIONS as usize);
                                completed += 1;
                            }
                            Err(cause) if memory_exhausted(&cause) && refused < 1_000 => {
                                refused += 1;
                                std::thread::yield_now();
                            }
                            Err(cause) => panic!("concurrent scan failed: {cause:?}"),
                        }
                    }
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
    });
    let settled = pressure(&runtime);
    assert!(settled.head_resident_bytes <= ceiling);
    assert_eq!(settled.head_borrowed_bytes, 0);
    assert!(settled.evictions_completed >= u64::from(REGIONS));
}
