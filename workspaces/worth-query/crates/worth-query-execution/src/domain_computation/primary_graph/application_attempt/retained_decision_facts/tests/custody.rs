use super::super::{
    directory::RunDirectory, run::Run, RetainedFact, RetainedFactStore, Slot, StoreDenial,
    AUTHOR_CHUNK,
};
use super::control;
use super::{key, merge, Body};
use std::{
    alloc::Layout,
    cell::RefCell,
    num::NonZeroUsize,
    sync::{Mutex, OnceLock},
};
use worth_execution::{
    CancellationToken, ExecutionAllocationDenialKind as Kind, ExecutionAllocationPolicy as Policy,
    ExecutionAuthority, ExecutionAuthorityConfig, LeaseDenial, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
pub(in crate::domain_computation::primary_graph) static SERIAL: Mutex<()> = Mutex::new(());
pub(in crate::domain_computation::primary_graph) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: None,
        })
        .unwrap()
    })
}
pub(in crate::domain_computation::primary_graph) fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), bytes, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}
fn layout<T>(count: usize) -> u64 {
    u64::try_from(Layout::array::<T>(count).unwrap().size()).unwrap()
}
fn starting_quote() -> u64 {
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
    let denied = match empty.insert(
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
    assert_eq!(denied.kind(), Kind::Lease(LeaseDenial::ResourceExhausted));
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
        *run.slots[0].borrow_mut() = Some(RetainedFact {
            key: key(locator, None, control(Policy::SystemAllocation)),
            value: Body {
                scalar: ordinal as u64,
                roles: 1,
            },
            ordinal,
        });
        run.used = 1;
    }
    let denied = match Run::merge(left, right, policy) {
        Err(StoreDenial::Allocation(denied)) => denied,
        _ => panic!("old+new coexistence must require a second actual layout"),
    };
    assert_eq!(denied.kind(), Kind::Lease(LeaseDenial::ResourceExhausted));
    assert_eq!(denied.requested_payload_bytes(), Some(quote));
    assert_eq!(parent.reserve_memory(quote).unwrap().charged_bytes(), quote);
    // Directory growth has the same old+replacement custody contract.
    let directory_quote = layout::<RefCell<Option<Run<Body>>>>(2);
    let directory_lease = authority().request_lease(request(directory_quote)).unwrap();
    let mut directory =
        RunDirectory::<Body>::new(control(Policy::Execution(&directory_lease))).unwrap();
    let denied = match directory.ensure(1, control(Policy::Execution(&directory_lease))) {
        Err(StoreDenial::Allocation(denied)) => denied,
        _ => panic!("replacement fits alone; coexistence must still require the old layout"),
    };
    assert_eq!(denied.kind(), Kind::Lease(LeaseDenial::ResourceExhausted));
    assert_eq!(denied.requested_payload_bytes(), Some(directory_quote));
    assert_eq!(directory.slots.as_ref().unwrap().len(), 1);
    drop(directory);
    assert_eq!(
        directory_lease
            .reserve_memory(directory_quote)
            .unwrap()
            .charged_bytes(),
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
                .insert(
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
        store.finish(policy).unwrap()
    };
    assert_eq!(records.charged_payload_bytes(), Some(final_quote));
    assert!(matches!(
        parent.reserve_memory(budget - final_quote - 2),
        Err(LeaseDenial::ResourceExhausted)
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
        Err(LeaseDenial::ResourceExhausted)
    ));
    drop(records);
    assert_eq!(
        parent.reserve_memory(budget).unwrap().charged_bytes(),
        budget
    );
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
    let stop = CancellationToken::new();
    let child = parent.controlled_child(stop.clone(), None);
    let policy = control(Policy::Execution(&child));
    let mut store = RetainedFactStore::new(policy).unwrap();
    let incoming = key(b"", None, policy);
    stop.cancel();
    let denied = store.insert(
        incoming,
        Body {
            scalar: 1,
            roles: 1,
        },
        policy,
        merge,
    );
    assert!(
        matches!(denied, Err(StoreDenial::Allocation(ref denied)) if denied.kind() == Kind::Cancelled)
    );
    assert!(matches!(
        store.finish(policy),
        Err(StoreDenial::Allocation(ref denial)) if denial.kind() == Kind::Cancelled
    ));
    assert!(!parent_stop.is_cancelled());
    assert_eq!(
        parent.reserve_memory(budget).unwrap().charged_bytes(),
        budget
    );
}

pub(super) fn isolated(name: &str, module: &str) -> bool {
    let full = format!("{module}::{name}");
    let filter = full.split_once("::").unwrap().1;
    if std::env::var("WORTH_QUERY_RETAINED_FACT_CASE").as_deref() == Ok(filter) {
        return true;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", filter, "--test-threads=1", "--nocapture"])
        .env("WORTH_QUERY_RETAINED_FACT_CASE", filter)
        .output()
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
