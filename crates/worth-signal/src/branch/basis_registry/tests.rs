use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;

use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

use super::super::retention::SignalBranchRetentionRegistry;
use super::super::{signal_branch_observation, SignalBranchRetentionAcquisitionDenial};
use super::*;

fn observation(branch_id: u64) -> SignalBranchObservation {
    signal_branch_observation(
        "basis-registry-single-flight",
        branch_id,
        format!("branch-{branch_id}"),
        FoundationalBranchTarget::empty(),
        FoundationalBranchReferenceGeneration::initial(),
    )
    .expect("test observation is valid")
}

fn wait_for_release(gate: &(Mutex<bool>, Condvar)) {
    let (open, wake) = gate;
    let mut open = open.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    while !*open {
        open = wake
            .wait(open)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
}

fn release(gate: &(Mutex<bool>, Condvar)) {
    let (open, wake) = gate;
    *open.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
    wake.notify_all();
}

#[test]
fn equal_callers_share_one_claimant_basis_and_owner_lease() {
    let registry = SignalBranchBasisRegistry::new();
    let retention = Arc::new(SignalBranchRetentionRegistry::new(17));
    let callback_count = Arc::new(AtomicUsize::new(0));
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let (claimed_tx, claimed_rx) = mpsc::sync_channel(1);

    let claimant_registry = registry.clone();
    let claimant_retention = Arc::clone(&retention);
    let claimant_count = Arc::clone(&callback_count);
    let claimant_gate = Arc::clone(&gate);
    let claimant = thread::spawn(move || {
        claimant_registry.admit_with_retention(
            31,
            7,
            crate::state::SignalBranchId(1),
            2,
            observation(1),
            |_| Ok(()),
            move || {
                claimant_count.fetch_add(1, Ordering::SeqCst);
                claimed_tx.send(()).expect("claimant announces its contact");
                wait_for_release(&claimant_gate);
                claimant_retention.acquire_admitted(crate::state::SignalBranchId(1))
            },
        )
    });
    claimed_rx
        .recv()
        .expect("the claimant is blocked after installing Acquiring");

    let mut waiters = Vec::new();
    for _ in 0..4 {
        let waiter_registry = registry.clone();
        waiters.push(thread::spawn(move || {
            waiter_registry.admit_with_retention(
                31,
                7,
                crate::state::SignalBranchId(1),
                2,
                observation(1),
                |_| Ok(()),
                || panic!("a single-flight waiter must not contact retention"),
            )
        }));
    }
    test_completion(
        &registry,
        &SignalBranchBasisRegistryKey::new(
            31,
            7,
            crate::state::SignalBranchId(1),
            2,
            &observation(1),
        ),
    )
    .expect("the claimant completion is installed")
    .wait_for_joiners(4);
    release(&gate);

    let basis = claimant
        .join()
        .expect("claimant thread remains healthy")
        .expect("claimant obtains the one real owner lease");
    let waiter_bases = waiters
        .into_iter()
        .map(|waiter| {
            waiter
                .join()
                .expect("single-flight waiter remains healthy")
                .expect("waiter receives the claimant basis")
        })
        .collect::<Vec<_>>();
    for waiter_basis in &waiter_bases {
        assert_eq!(
            basis.admission_identity(),
            waiter_basis.admission_identity(),
            "all equal callers receive the one owner-issued identity"
        );
    }
    assert_eq!(callback_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        retention.admitted_count(crate::state::SignalBranchId(1)),
        1,
        "one canonical basis owns one admitted retention lease"
    );

    drop(waiter_bases);
    drop(basis);
    assert_eq!(
        retention.admitted_count(crate::state::SignalBranchId(1)),
        0,
        "weak canonicalization does not retain the component lease"
    );
}

#[test]
fn ready_reuse_validates_outside_lock_without_contacting_retention() {
    let registry = SignalBranchBasisRegistry::new();
    let retention = Arc::new(SignalBranchRetentionRegistry::new(29));
    let canonical = registry
        .admit_with_retention(
            71,
            15,
            crate::state::SignalBranchId(6),
            7,
            observation(6),
            |_| Ok(()),
            || retention.acquire_admitted(crate::state::SignalBranchId(6)),
        )
        .expect("the first exact basis installs the canonical owner lease");
    let callback_registry = registry.clone();
    let reused = registry
        .admit_with_retention(
            71,
            15,
            crate::state::SignalBranchId(6),
            7,
            observation(6),
            |existing| {
                let nested = callback_registry
                    .admit_with_retention(
                        71,
                        15,
                        crate::state::SignalBranchId(6),
                        7,
                        observation(6),
                        |_| Ok(()),
                        || panic!("ready reuse must not contact retention"),
                    )
                    .expect("ready validation runs after the registry lock is released");
                assert_eq!(nested.admission_identity(), existing.admission_identity());
                Ok(())
            },
            || panic!("ready reuse must not acquire a second owner lease"),
        )
        .expect("the owner validation permits the live canonical basis");
    assert_eq!(reused.admission_identity(), canonical.admission_identity());
    assert_eq!(retention.admitted_count(crate::state::SignalBranchId(6)), 1);

    let denial = SignalBranchRetentionAcquisitionDenial::RetiredBranch {
        branch_id: crate::state::SignalBranchId(6),
    };
    let ready_denial = registry.admit_with_retention(
        71,
        15,
        crate::state::SignalBranchId(6),
        7,
        observation(6),
        |_| Err(denial.clone()),
        || panic!("a denied ready reuse must not contact retention"),
    );
    assert!(matches!(ready_denial, Err(actual) if actual == denial));
    assert_eq!(retention.admitted_count(crate::state::SignalBranchId(6)), 1);
    let retry = registry
        .admit_with_retention(
            71,
            15,
            crate::state::SignalBranchId(6),
            7,
            observation(6),
            |_| Ok(()),
            || panic!("a healthy retry still reuses the canonical basis"),
        )
        .expect("a denied validation does not evict the live canonical basis");
    assert_eq!(retry.admission_identity(), canonical.admission_identity());
}

#[test]
fn claimant_denial_is_shared_and_reservation_allows_a_later_retry() {
    let registry = SignalBranchBasisRegistry::new();
    let retention = Arc::new(SignalBranchRetentionRegistry::new(19));
    let callback_count = Arc::new(AtomicUsize::new(0));
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let (claimed_tx, claimed_rx) = mpsc::sync_channel(1);
    let denial = SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
        maximum_active_leases: 1,
    };

    let claimant_registry = registry.clone();
    let claimant_count = Arc::clone(&callback_count);
    let claimant_gate = Arc::clone(&gate);
    let claimant_denial = denial.clone();
    let claimant = thread::spawn(move || {
        claimant_registry.admit_with_retention(
            41,
            9,
            crate::state::SignalBranchId(2),
            3,
            observation(2),
            |_| Ok(()),
            move || {
                claimant_count.fetch_add(1, Ordering::SeqCst);
                claimed_tx.send(()).expect("claimant announces its contact");
                wait_for_release(&claimant_gate);
                Err(claimant_denial)
            },
        )
    });
    claimed_rx
        .recv()
        .expect("the denial claimant is blocked after installing Acquiring");

    let mut waiters = Vec::new();
    for _ in 0..4 {
        let waiter_registry = registry.clone();
        waiters.push(thread::spawn(move || {
            waiter_registry.admit_with_retention(
                41,
                9,
                crate::state::SignalBranchId(2),
                3,
                observation(2),
                |_| Ok(()),
                || panic!("a denial waiter must not make a second owner contact"),
            )
        }));
    }
    test_completion(
        &registry,
        &SignalBranchBasisRegistryKey::new(
            41,
            9,
            crate::state::SignalBranchId(2),
            3,
            &observation(2),
        ),
    )
    .expect("the denial completion is installed")
    .wait_for_joiners(4);
    release(&gate);

    assert!(matches!(
        claimant.join().expect("claimant remains healthy"),
        Err(actual) if actual == denial
    ));
    for waiter in waiters {
        assert!(matches!(
            waiter.join().expect("denial waiter remains healthy"),
            Err(actual) if actual == denial
        ));
    }
    assert_eq!(callback_count.load(Ordering::SeqCst), 1);

    let retry = registry
        .admit_with_retention(
            41,
            9,
            crate::state::SignalBranchId(2),
            3,
            observation(2),
            |_| Ok(()),
            || retention.acquire_admitted(crate::state::SignalBranchId(2)),
        )
        .expect("a denied reservation is removed for a healthy retry");
    assert_eq!(retention.admitted_count(crate::state::SignalBranchId(2)), 1);
    drop(retry);
}

#[test]
fn same_key_reentry_denies_immediately_but_different_key_reentry_is_valid() {
    let registry = SignalBranchBasisRegistry::new();
    let retention = Arc::new(SignalBranchRetentionRegistry::new(23));
    let nested_registry = registry.clone();
    let nested_retention = Arc::clone(&retention);
    let basis = registry
        .admit_with_retention(
            51,
            11,
            crate::state::SignalBranchId(3),
            4,
            observation(3),
            |_| Ok(()),
            move || {
                let same_key = nested_registry.admit_with_retention(
                    51,
                    11,
                    crate::state::SignalBranchId(3),
                    4,
                    observation(3),
                    |_| Ok(()),
                    || panic!("same-key reentry must not invoke retention"),
                );
                assert!(matches!(
                    same_key,
                    Err(SignalBranchRetentionAcquisitionDenial::OwnerReentry)
                ));
                let different_key = nested_registry.admit_with_retention(
                    51,
                    11,
                    crate::state::SignalBranchId(4),
                    4,
                    observation(4),
                    |_| Ok(()),
                    move || nested_retention.acquire_admitted(crate::state::SignalBranchId(4)),
                )?;
                drop(different_key);
                retention.acquire_admitted(crate::state::SignalBranchId(3))
            },
        )
        .expect("different-key owner reentry remains valid");
    drop(basis);
}

#[test]
fn panicking_claimant_unwinds_reservation_and_releases_waiters() {
    let registry = SignalBranchBasisRegistry::new();
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let (claimed_tx, claimed_rx) = mpsc::sync_channel(1);
    let claimant_registry = registry.clone();
    let claimant_gate = Arc::clone(&gate);
    let claimant = thread::spawn(move || {
        claimant_registry.admit_with_retention(
            61,
            13,
            crate::state::SignalBranchId(5),
            6,
            observation(5),
            |_| Ok(()),
            move || {
                claimed_tx.send(()).expect("claimant announces its contact");
                wait_for_release(&claimant_gate);
                panic!("test claimant unwinds")
            },
        )
    });
    claimed_rx
        .recv()
        .expect("the panicking claimant installed Acquiring");
    let waiter_registry = registry.clone();
    let waiter = thread::spawn(move || {
        waiter_registry.admit_with_retention(
            61,
            13,
            crate::state::SignalBranchId(5),
            6,
            observation(5),
            |_| Ok(()),
            || panic!("panic cleanup waiter must not contact retention"),
        )
    });
    test_completion(
        &registry,
        &SignalBranchBasisRegistryKey::new(
            61,
            13,
            crate::state::SignalBranchId(5),
            6,
            &observation(5),
        ),
    )
    .expect("the panic completion is installed")
    .wait_for_joiners(1);
    release(&gate);
    assert!(claimant.join().is_err());
    assert!(matches!(
        waiter.join().expect("panic cleanup waiter remains healthy"),
        Err(SignalBranchRetentionAcquisitionDenial::OwnerOperationPanicked)
    ));
}
