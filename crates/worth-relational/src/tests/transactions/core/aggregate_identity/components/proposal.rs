//! Real custom planning/evaluation shares the same projected component and original payer.
use super::super::read_handoff_rule::{registration, Control, ReadMode};
use super::fixtures::*;
use std::sync::Arc;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn component_proposed_and_committed_rule_reads_share_sealed_identity_and_last_drop_custody() {
    let memory = SerialMemoryBudget::new(Some(128 * 1024 * 1024));
    let host = SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let execution = &ExecutionRequest::serial(&host);
    let registry = registry();
    let source = registry.relation_kinds[&KindId(7)]
        .relation_integrity
        .aggregate_identity
        .clone();
    let control = Arc::new(Control {
        source,
        ..Control::default()
    });
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(registration(10, ReadMode::Both, None, &control))
        .custom_invariant(registration(11, ReadMode::Both, None, &control))
        .build();
    let born = commit(execution, &runtime, birth());
    let ids = ids(&born);
    let old = read(execution, &runtime, &born.snapshot, false, ids[1]);
    let old_identity = old.identity();
    drop(old);
    control.arm(vec![ids[1]]);
    let split = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("rule-component-split")
            .push(delete_edge(&born, 0, 2))
            .push(delete_edge(&born, 1, 6))
            .push(delete_edge(&born, 5, 6)),
    );
    let published = read(execution, &runtime, &split.snapshot, false, ids[1]);
    assert_ne!(published.identity(), old_identity);
    let survivor = {
        let captures = control.captures.lock().unwrap();
        assert_eq!(
            captures
                .iter()
                .map(|capture| (capture.rule, capture.evaluation))
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from([(10, false), (10, true), (11, false), (11, true)])
        );
        for capture in captures.iter() {
            assert!(capture.errors.is_empty(), "{:?}", capture.errors);
            assert!(!capture.proposed.is_empty());
            assert!(!capture.committed.is_empty());
            for value in &capture.proposed {
                assert_eq!(value.identity(), published.identity());
                assert_eq!(value.members().collect::<Vec<_>>(), ids[1..3]);
                assert_eq!(value.transition_lineage().count(), 1);
            }
            for value in &capture.committed {
                assert_eq!(value.identity(), old_identity);
                assert_eq!(value.members().collect::<Vec<_>>(), ids[..3]);
            }
        }
        captures[0].proposed[0].clone()
    };
    drop(published);
    release_test_commit_snapshot(&runtime, &born);
    release_test_commit_snapshot(&runtime, &split);
    control.captures.lock().unwrap().clear();
    assert!(memory.charged_memory_bytes() > 0);
    assert_eq!(survivor.members().collect::<Vec<_>>(), ids[1..3]);
    drop(survivor);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
