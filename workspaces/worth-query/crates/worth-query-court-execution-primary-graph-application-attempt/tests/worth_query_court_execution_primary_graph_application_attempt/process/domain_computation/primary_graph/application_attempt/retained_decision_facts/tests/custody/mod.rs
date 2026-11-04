use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::directory::RunDirectory;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::run::Run;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::RetainedFact;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::storage::RetainedFactStore;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::Slot;
use worth_query_execution::facade::application_inspection::StoreDenial;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::AUTHOR_CHUNK;
use crate::process::domain_computation::primary_graph::application_attempt::retained_decision_facts::tests::control;
use crate::process::domain_computation::primary_graph::application_attempt::retained_decision_facts::tests::key;
use crate::process::domain_computation::primary_graph::application_attempt::retained_decision_facts::tests::merge;
use crate::process::domain_computation::primary_graph::application_attempt::retained_decision_facts::tests::Body;
use std::alloc::Layout;
use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::sync::OnceLock;
use worth_execution::CancellationToken;
use worth_execution::ExecutionAllocationDenialClass as Kind;
use worth_execution::ExecutionAllocationPolicy as Policy;
use worth_execution::ExecutionAuthority;
use worth_execution::ExecutionAuthorityConfig;
use worth_execution::LeaseDenial;
use worth_execution::LeaseRequest;
use worth_foundational::DeterminismContract;
use worth_foundational::ExecutionBudget;
use worth_foundational::ExecutionPosture;
use worth_foundational::ExecutionRequestPolicy;

pub static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
pub static SERIAL: Mutex<()> = Mutex::new(());
pub fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: None,
        })
        .unwrap()
    })
}
pub fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(bytes), Some(1)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}
pub fn layout<T>(count: usize) -> u64 {
    u64::try_from(Layout::array::<T>(count).unwrap().size()).unwrap()
}
pub fn starting_quote() -> u64 {
    layout::<Slot<Body>>(AUTHOR_CHUNK)
}

#[test]
fn before_growth_refusal_and_merge_replacement_coexistence_release_exact_backings() {
    if !isolated(
        "before_growth_refusal_and_merge_replacement_coexistence_release_exact_backings",
        module_path!(),
    ) {
        return;
    }
    let _serial = SERIAL.lock().unwrap();
    let zero = authority().request_lease(request(0)).unwrap();
    let mut empty = RetainedFactStore::<Body>::new(control(Policy::Execution(&zero))).unwrap();
    let denied = match empty.insert_for_test(
        key(b"x", None, control(Policy::SystemAllocation)),
        Body {
            scalar: 1,
            roles: 1,
        },
        control(Policy::Execution(&zero)),
        merge,
    ) {
        Err(StoreDenial::Allocation(denied)) => denied,
        _ => panic!("first nonzero chunk must require its actual layout"),
    };
    assert_eq!(
        denied.classify(),
        Kind::Request(worth_execution::WorkCeilingDenial::Admission(
            LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                requested: starting_quote(),
                admitted: 0,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
            })
        ))
    );
    assert_eq!(
        denied.requested_payload_bytes(),
        Some(layout::<Slot<Body>>(AUTHOR_CHUNK))
    );
    let quote = layout::<Slot<Body>>(2);
    let parent = authority().request_lease(request(quote)).unwrap();
    let policy = control(Policy::Execution(&parent));
    let mut left = Run::empty(1, policy).unwrap();
    let mut right = Run::empty(1, policy).unwrap();
    for (run, locator, ordinal) in [(&mut left, b"a", 0), (&mut right, b"b", 1)] {
        // Explicit system key bytes excluded in this inline merge proof.
        *(*run.slots_for_test())[0].borrow_mut() = Some(RetainedFact {
            key: key(locator, None, control(Policy::SystemAllocation)),
            value: Body {
                scalar: ordinal as u64,
                roles: 1,
            },
            ordinal,
        });
        (*run.used_mut_for_test()) = 1;
    }
    let denied = match Run::merge_for_test(left, right, policy) {
        Err(StoreDenial::Allocation(denied)) => denied,
        _ => panic!("old+new coexistence must require a second actual layout"),
    };
    assert_eq!(
        denied.classify(),
        Kind::Request(worth_execution::WorkCeilingDenial::Admission(
            LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                requested: quote,
                admitted: 0,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
            })
        ))
    );
    assert_eq!(denied.requested_payload_bytes(), Some(quote));
    assert_eq!(parent.reserve_memory(quote).unwrap().bytes(), quote);
    // Directory growth has the same old+replacement custody contract.
    let directory_quote = layout::<RefCell<Option<Run<Body>>>>(2);
    let directory_lease = authority().request_lease(request(directory_quote)).unwrap();
    let mut directory =
        RunDirectory::<Body>::new(control(Policy::Execution(&directory_lease))).unwrap();
    let denied = match directory.ensure_for_test(1, control(Policy::Execution(&directory_lease))) {
        Err(StoreDenial::Allocation(denied)) => denied,
        _ => panic!("replacement fits alone; coexistence must still require the old layout"),
    };
    assert_eq!(
        denied.classify(),
        Kind::Request(worth_execution::WorkCeilingDenial::Admission(
            LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                requested: directory_quote,
                admitted: directory_quote - layout::<RefCell<Option<Run<Body>>>>(1),
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
            })
        ))
    );
    assert_eq!(denied.requested_payload_bytes(), Some(directory_quote));
    assert_eq!(directory.slots.as_ref().unwrap().len(), 1);
    drop(directory);
    assert_eq!(
        directory_lease
            .reserve_memory(directory_quote)
            .unwrap()
            .bytes(),
        directory_quote
    );
}

#[test]
fn final_array_and_key_charges_survive_child_drop_and_exhausted_iterator() {
    if !isolated(
        "final_array_and_key_charges_survive_child_drop_and_exhausted_iterator",
        module_path!(),
    ) {
        return;
    }
    let _serial = SERIAL.lock().unwrap();
    let final_quote = layout::<RetainedFact<Body>>(3);
    let budget = starting_quote() + final_quote + 3; // Three one-byte test locators.
    let parent = authority().request_lease(request(budget)).unwrap();
    let records = {
        let child = parent.child(request(budget)).unwrap();
        let policy = control(Policy::Execution(&child));
        let mut store = RetainedFactStore::new(policy).unwrap();
        for (ordinal, locator) in [b"c", b"a", b"b"].into_iter().enumerate() {
            store
                .insert_for_test(
                    key(locator, None, policy),
                    Body {
                        scalar: ordinal as u64,
                        roles: 1,
                    },
                    policy,
                    merge,
                )
                .unwrap();
        }
        store.finish_for_test(policy).unwrap()
    };
    assert_eq!(records.charged_payload_bytes(), Some(final_quote));
    assert!(matches!(
        parent.reserve_memory(budget - final_quote - 2),
        Err(memory) if memory == worth_execution::MemoryLimitDenial::Capacity { requested: budget - final_quote - 2, admitted: budget - final_quote - 3, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }
    ));
    drop(parent.reserve_memory(budget - final_quote - 3).unwrap());
    let mut records = records.into_iter();
    assert_eq!(records.next().unwrap().key.locator(), b"a");
    for record in records.by_ref() {
        drop(record);
    }
    assert_eq!(records.charged_payload_bytes(), Some(final_quote));
    assert!(matches!(
        parent.reserve_memory(budget - final_quote + 1),
        Err(memory) if memory == worth_execution::MemoryLimitDenial::Capacity { requested: budget - final_quote + 1, admitted: budget - final_quote, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }
    ));
    drop(records);
    assert_eq!(parent.reserve_memory(budget).unwrap().bytes(), budget);
}

#[test]
fn live_child_stop_refuses_partial_success_without_stopping_parent() {
    if !isolated(
        "live_child_stop_refuses_partial_success_without_stopping_parent",
        module_path!(),
    ) {
        return;
    }
    let _serial = SERIAL.lock().unwrap();
    let budget = starting_quote();
    let parent_request = request(budget);
    let parent_stop = parent_request.cancellation.clone();
    let parent = authority().request_lease(parent_request).unwrap();
    let stop = worth_execution::CancellationSource::new();
    let child = parent.controlled_child(stop.token(), None);
    let policy = control(Policy::Execution(&child));
    let mut store = RetainedFactStore::new(policy).unwrap();
    let incoming = key(b"", None, policy);
    stop.cancel();
    let denied = store.insert_for_test(
        incoming,
        Body {
            scalar: 1,
            roles: 1,
        },
        policy,
        merge,
    );
    assert!(
        matches!(denied, Err(StoreDenial::Allocation(ref denied)) if denied.classify() == Kind::Request(worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)))
    );
    assert!(matches!(
        store.finish_for_test(policy),
        Err(StoreDenial::Allocation(ref denial)) if denial.classify() == Kind::Request(worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled))
    ));
    assert!(!parent_stop.is_cancelled());
    assert_eq!(parent.reserve_memory(budget).unwrap().bytes(), budget);
}

pub fn isolated(name: &str, module: &str) -> bool {
    let full = format!("{module}::{name}");
    let filter = full.split_once("::").unwrap().1;
    if std::env::var("WORTH_QUERY_RETAINED_FACT_CASE").as_deref() == Ok(filter) {
        return true;
    }
    let output = worth_query_testkit_execution_primary_graph_application_attempt_2::process_deadline::output(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", filter, "--test-threads=1", "--nocapture"])
            .env("WORTH_QUERY_RETAINED_FACT_CASE", filter),
    )
    .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
    false
}
