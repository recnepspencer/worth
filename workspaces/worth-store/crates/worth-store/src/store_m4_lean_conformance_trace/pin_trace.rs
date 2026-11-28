//! Conformance trace emitter for `StoreM4.PinSafety` and `StoreM4.GCPinSafety`,
//! driven by the real compiled `PinTableKernel` and `VolatileRootPinTable`.

use super::Trace;
use crate::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinState, VolatileRootPinTable};
use crate::pin_table_kernel::{PinLifecycleState, PinRecord, PinTableKernel};

fn state_str(state: PinLifecycleState) -> &'static str {
    match state {
        PinLifecycleState::Active => "active",
        PinLifecycleState::Expiring => "expiring",
        PinLifecycleState::Expired => "expired",
    }
}

fn entry_json(record: PinRecord) -> String {
    let parent = record
        .parent_pin_id
        .map_or_else(|| "null".to_owned(), |pid| pid.to_string());
    format!(
        r#"{{"id":{},"root":{},"parent":{parent},"state":"{}","loans":{},"children":{}}}"#,
        record.pin_id,
        record.root_id,
        state_str(record.state),
        record.active_reader_loans,
        record.live_child_pins,
    )
}

fn table_json(kernel: &PinTableKernel, ids: &[u64]) -> String {
    let rows: Vec<String> = ids
        .iter()
        .filter_map(|&id| kernel.record(id).map(entry_json))
        .collect();
    format!("[{}]", rows.join(","))
}

fn probes_json(kernel: &PinTableKernel, roots: &[u64]) -> String {
    let rows: Vec<String> = roots
        .iter()
        .map(|&root| format!("[{root},{}]", kernel.is_root_protected(root)))
        .collect();
    format!("[{}]", rows.join(","))
}

fn record_pin_safety_expire_steps(trace: &mut Trace) {
    let mut kernel = PinTableKernel::new();
    let ids = [10, 11];
    let roots = [100, 200, 300];
    kernel.acquire_pin(10, 100, None).unwrap();
    kernel.acquire_pin(11, 200, None).unwrap();
    kernel.acquire_reader_loan(10).unwrap();

    // 1. Active -> Expiring while loan held.
    let before = table_json(&kernel, &ids);
    let next = kernel.expire_lease(10).unwrap();
    assert_eq!(next, PinLifecycleState::Expiring);
    let after = table_json(&kernel, &ids);
    trace.push(
        "pin_expire_step",
        format!(
            r#""before":{before},"target_id":10,"after":{after},"probes":{}"#,
            probes_json(&kernel, &roots)
        ),
    );

    // 2. Expiring stays Expiring while active_reader_loans > 0; root 100 stays protected.
    let before = table_json(&kernel, &ids);
    let next = kernel.expire_lease(10).unwrap();
    assert_eq!(next, PinLifecycleState::Expiring);
    assert!(kernel.is_root_protected(100));
    let after = table_json(&kernel, &ids);
    trace.push(
        "pin_expire_step",
        format!(
            r#""before":{before},"target_id":10,"after":{after},"probes":{}"#,
            probes_json(&kernel, &roots)
        ),
    );

    // 3. Release loan, then Expiring -> Expired; root 100 no longer protected while root 200 stays protected.
    kernel.release_reader_loan(10).unwrap();
    let before = table_json(&kernel, &ids);
    let next = kernel.expire_lease(10).unwrap();
    assert_eq!(next, PinLifecycleState::Expired);
    assert!(!kernel.is_root_protected(100));
    assert!(kernel.is_root_protected(200));
    let after = table_json(&kernel, &ids);
    trace.push(
        "pin_expire_step",
        format!(
            r#""before":{before},"target_id":10,"after":{after},"probes":{}"#,
            probes_json(&kernel, &roots)
        ),
    );

    // 4. Expired -> Expired idempotent step.
    let before = table_json(&kernel, &ids);
    let next = kernel.expire_lease(10).unwrap();
    assert_eq!(next, PinLifecycleState::Expired);
    let after = table_json(&kernel, &ids);
    trace.push(
        "pin_expire_step",
        format!(
            r#""before":{before},"target_id":10,"after":{after},"probes":{}"#,
            probes_json(&kernel, &roots)
        ),
    );
}

fn push_gc_acquire(
    trace: &mut Trace,
    kernel: &mut PinTableKernel,
    ids: &mut Vec<u64>,
    roots: &[u64],
    id: u64,
    root: u64,
    parent: Option<u64>,
) {
    let before = table_json(kernel, ids);
    let ok = kernel.acquire_pin(id, root, parent).is_ok();
    if ok && !ids.contains(&id) {
        ids.push(id);
    }
    let after = table_json(kernel, ids);
    let parent_json = parent.map_or_else(|| "null".to_owned(), |p| p.to_string());
    trace.push(
        "gc_pin_transition",
        format!(
            r#""op":"acquire","id":{id},"root":{root},"parent":{parent_json},"ok":{ok},"before":{before},"after":{after},"probes":{}"#,
            probes_json(kernel, roots)
        ),
    );
}

fn push_gc_unary(
    trace: &mut Trace,
    kernel: &mut PinTableKernel,
    ids: &[u64],
    roots: &[u64],
    op: &str,
    id: u64,
    run: impl FnOnce(&mut PinTableKernel) -> bool,
) {
    let before = table_json(kernel, ids);
    let ok = run(kernel);
    let after = table_json(kernel, ids);
    trace.push(
        "gc_pin_transition",
        format!(
            r#""op":"{op}","id":{id},"ok":{ok},"before":{before},"after":{after},"probes":{}"#,
            probes_json(kernel, roots)
        ),
    );
}

fn record_gc_pin_safety_transitions(trace: &mut Trace) {
    let mut kernel = PinTableKernel::new();
    let mut ids = Vec::new();
    let roots = [10, 20, 30];

    // Acquire parent, duplicate refusal, missing-parent refusal, acquire child overlay.
    push_gc_acquire(trace, &mut kernel, &mut ids, &roots, 1, 10, None);
    push_gc_acquire(trace, &mut kernel, &mut ids, &roots, 1, 20, None);
    push_gc_acquire(trace, &mut kernel, &mut ids, &roots, 2, 20, Some(99));
    push_gc_acquire(trace, &mut kernel, &mut ids, &roots, 2, 20, Some(1));

    // Reader loan on active child, then expire parent (Active -> Expiring).
    push_gc_unary(trace, &mut kernel, &ids, &roots, "loan_acquire", 2, |k| {
        k.acquire_reader_loan(2).is_ok()
    });
    push_gc_unary(trace, &mut kernel, &ids, &roots, "expire", 1, |k| {
        k.expire_lease(1).is_ok()
    });

    // Fenced operations on Expiring parent: loan refused, child acquire refused,
    // repeated expire keeps parent Expiring while child is live.
    push_gc_unary(trace, &mut kernel, &ids, &roots, "loan_acquire", 1, |k| {
        k.acquire_reader_loan(1).is_ok()
    });
    push_gc_acquire(trace, &mut kernel, &mut ids, &roots, 3, 30, Some(1));
    push_gc_unary(trace, &mut kernel, &ids, &roots, "expire", 1, |k| {
        k.expire_lease(1).is_ok()
    });

    // Expire child (Active -> Expiring), loan refused on Expiring child,
    // repeated expire held by loan.
    push_gc_unary(trace, &mut kernel, &ids, &roots, "expire", 2, |k| {
        k.expire_lease(2).is_ok()
    });
    push_gc_unary(trace, &mut kernel, &ids, &roots, "loan_acquire", 2, |k| {
        k.acquire_reader_loan(2).is_ok()
    });
    push_gc_unary(trace, &mut kernel, &ids, &roots, "expire", 2, |k| {
        k.expire_lease(2).is_ok()
    });

    // Release loan (and saturating release at 0 + missing pin refusal).
    push_gc_unary(trace, &mut kernel, &ids, &roots, "loan_release", 2, |k| {
        k.release_reader_loan(2).is_ok()
    });
    push_gc_unary(trace, &mut kernel, &ids, &roots, "loan_release", 2, |k| {
        k.release_reader_loan(2).is_ok()
    });
    push_gc_unary(trace, &mut kernel, &ids, &roots, "loan_release", 99, |k| {
        k.release_reader_loan(99).is_ok()
    });

    // Child completes Expiring -> Expired (decrementing parent live_child_pins),
    // then parent completes Expiring -> Expired, then missing-pin expire refused.
    push_gc_unary(trace, &mut kernel, &ids, &roots, "expire", 2, |k| {
        k.expire_lease(2).is_ok()
    });
    push_gc_unary(trace, &mut kernel, &ids, &roots, "expire", 1, |k| {
        k.expire_lease(1).is_ok()
    });
    push_gc_unary(trace, &mut kernel, &ids, &roots, "expire", 99, |k| {
        k.expire_lease(99).is_ok()
    });
}

fn verify_volatile_root_pin_table_wrapper() {
    let table = VolatileRootPinTable::default();
    let parent = table
        .pin(
            10,
            None,
            RootPinDropPolicy::RetainUntilChildrenDrain,
            vec![1],
        )
        .unwrap();
    let child = table
        .pin(
            20,
            Some(parent),
            RootPinDropPolicy::RefuseIfPinnedByChild,
            vec![2],
        )
        .unwrap();
    assert_eq!(table.loan(child).unwrap(), vec![2]);
    assert_eq!(table.expire(parent).unwrap(), RootPinState::Expiring);
    assert_eq!(table.loan(parent), Err(RootPinDenial::LeaseExpired));
    assert!(table.protects_root(10));
    assert_eq!(table.expire(child).unwrap(), RootPinState::Expiring);
    assert!(table.protects_root(20));
    table.release_loan(child);
    assert_eq!(table.state(child), Some(RootPinState::Expired));
    assert_eq!(table.state(parent), Some(RootPinState::Expired));
    assert!(!table.protects_root(10));
    assert!(!table.protects_root(20));
}

pub(super) fn record(trace: &mut Trace) {
    record_pin_safety_expire_steps(trace);
    record_gc_pin_safety_transitions(trace);
    verify_volatile_root_pin_table_wrapper();
}
