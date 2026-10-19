//! Receiving-owner custody, not a synthetic payload-accounting observer.
use crate::symbols::data::RelationalFieldTextAdmissionDenial as Denial;
use crate::tests::support::{runtime_with_test_schema, test_execution_lease};
use std::{num::NonZeroUsize, sync::Arc};
use worth_execution::{CancellationToken, ExecutionImmutableBytes, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn request(memory: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), memory, Some(1_000_000)),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}
fn external(text: &str) -> ExecutionImmutableBytes {
    ExecutionImmutableBytes::from_external_bytes(Arc::new(text.as_bytes().into()))
}

#[test]
fn field_text_import_denial_preserves_source_table_and_head() {
    let runtime = runtime_with_test_schema();
    let identity = runtime.main_branch_identity();
    let (head, basis) = runtime.observe_branch(&identity).unwrap();
    let parent = test_execution_lease();
    let limited = parent.child(request(1)).unwrap();
    let source = external("native-actor-json");
    let before = runtime.services.symbols.snapshot();
    let denial = runtime
        .admit_field_text(&ExecutionRequest::leased(&limited), &basis, &source)
        .unwrap_err();
    assert!(matches!(denial, Denial::Allocation(_)));
    assert_eq!(source.bytes(), b"native-actor-json");
    assert_eq!(runtime.services.symbols.snapshot(), before);
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, head);
}

#[test]
fn field_text_upgrade_replaces_system_keys_and_last_snapshot_retains_charge() {
    let parent = test_execution_lease();
    let lease = parent.child(request(256)).unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let raw = "native-actor-json";
    let symbol = runtime
        .services
        .symbols
        .with_write(|symbols| symbols.intern(raw));
    let admitted = runtime
        .admit_field_text(&execution, &basis, &external(raw))
        .unwrap();
    assert_eq!(admitted.charged_payload_bytes(), Some(raw.len() as u64));
    assert!(admitted.field_value_for(&basis).is_ok());
    runtime.services.symbols.with_read(|symbols| {
        assert_eq!(symbols.symbol(raw), Some(symbol));
        assert!(symbols.retained_text(symbol).unwrap().is_managed());
        assert!(symbols
            .snapshot()
            .entries
            .iter()
            .find(|(_, text)| text.as_str() == raw)
            .unwrap()
            .1
            .is_managed());
    });
    let snapshot = runtime.services.symbols.configuration_snapshot();
    assert!(snapshot
        .entries
        .iter()
        .find(|(_, text)| text.as_str() == raw)
        .unwrap()
        .1
        .is_managed());
    drop(admitted);
    drop(runtime);
    assert!(
        lease.reserve_memory(256).is_err(),
        "actual snapshot still owns the reservation"
    );
    drop(snapshot);
    assert!(
        lease.reserve_memory(256).is_ok(),
        "last actual backing owner releases the charge"
    );
}

#[test]
fn serial_import_is_explicit_system_custody_and_foreign_basis_refuses() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let other = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let (_, foreign) = other.observe_branch(&other.main_branch_identity()).unwrap();
    assert!(matches!(
        runtime.admit_field_text(&execution, &foreign, &external("actor")),
        Err(Denial::ForeignBasis)
    ));
    let admitted = runtime
        .admit_field_text(&execution, &basis, &external("actor"))
        .unwrap();
    assert_eq!(admitted.charged_payload_bytes(), None);
    assert_eq!(admitted.as_str(), "actor");
    assert!(matches!(
        admitted.field_value_for(&foreign),
        Err(Denial::ForeignBasis)
    ));
    assert!(matches!(
        runtime.admit_field_text(
            &execution,
            &basis,
            &ExecutionImmutableBytes::from_external_bytes(Arc::new(vec![0xff].into_boxed_slice()))
        ),
        Err(Denial::InvalidUtf8)
    ));
}

#[test]
fn field_text_shares_actual_charged_input_including_empty_backing() {
    let parent = test_execution_lease();
    let lease = parent.child(request(256)).unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    for raw in ["already-funded", ""] {
        let mut bytes = worth_execution::ExecutionByteBuffer::allocate(
            raw.len(),
            execution.allocation_policy(),
        )
        .unwrap();
        bytes.extend_from_slice(raw.as_bytes()).unwrap();
        let input = bytes.seal().unwrap();
        let admitted = runtime
            .admit_field_text(&execution, &basis, &input)
            .unwrap();
        assert_eq!(admitted.charged_payload_bytes(), Some(raw.len() as u64));
        assert_eq!(admitted.as_str().as_ptr(), input.bytes().as_ptr());
        let remaining = 256 - "already-funded".len() as u64;
        assert!(
            lease.reserve_memory(remaining).is_ok(),
            "receiving owner did not fund a second copy"
        );
    }
}

#[test]
fn field_text_symbol_exhaustion_refuses_before_selected_policy_copy() {
    let mut config = runtime_with_test_schema().config();
    config.identity.symbol_table.entries = vec![(
        crate::symbols::data::Symbol(u32::MAX - 1),
        "last-symbol".into(),
    )];
    let runtime = crate::runtime::RelationalRuntime::new(config);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let parent = test_execution_lease();
    let lease = parent.child(request(1)).unwrap();
    let before = runtime.services.symbols.snapshot();
    assert!(matches!(
        runtime.admit_field_text(
            &ExecutionRequest::leased(&lease),
            &basis,
            &external("new-symbol")
        ),
        Err(Denial::SymbolExhausted)
    ));
    assert_eq!(runtime.services.symbols.snapshot(), before);
}

#[test]
fn field_text_cancelled_and_stale_receivers_preserve_table() {
    let parent = test_execution_lease();
    let cancel = worth_execution::CancellationSource::new();
    let mut selected = request(256);
    selected.cancellation = cancel.token();
    let lease = parent.child(selected).unwrap();
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let before = runtime.services.symbols.snapshot();
    cancel.cancel();
    assert!(matches!(
        runtime.admit_field_text(
            &ExecutionRequest::leased(&lease),
            &basis,
            &external("cancelled")
        ),
        Err(Denial::Stopped(worth_execution::MapKernelStop::Cancelled))
    ));
    assert_eq!(runtime.services.symbols.snapshot(), before);
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let admitted = runtime
        .admit_field_text(execution, &basis, &external("retained-after-movement"))
        .unwrap();
    crate::tests::support::create_entity(execution, &runtime, "native-new-head");
    let moved = runtime.services.symbols.snapshot();
    assert!(matches!(
        admitted.field_value_for(&basis),
        Err(Denial::StaleBasis)
    ));
    assert!(matches!(
        runtime.admit_field_text(execution, &basis, &external("must-not-intern")),
        Err(Denial::StaleBasis)
    ));
    assert_eq!(runtime.services.symbols.snapshot(), moved);
}
