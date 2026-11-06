//! Native lifecycle proof uses physical checkpoints/tail and an actual retained reader.
use super::{
    cold_envelope::{reopen_with_tail, Source},
    expected,
    fixture::grid,
    historical_custody::{locator, replace},
    selected_value::assert_selected,
    transaction::{batch, fixture, FORMAT},
};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::ContentRefId;
use worth_relational::facade::{
    content::RelationalContentDenial, durability::*, identity::EntityId,
    mvcc::RelationalTransactionIntent, transactions::RecordRef,
};

#[test]
fn checkpoint_tail_replacement_and_retained_history_keep_exact_native_content() {
    let cancel = CancellationSource::new();
    // Cold recovery reserves the bounded 256 MiB physical-read ceiling before
    // invoking its port. Fund that real original allowance and live custody.
    let budget = SerialMemoryBudget::new(Some(512 * 1024 * 1024));
    let request = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let before = budget.available_memory_bytes().unwrap();
    let execution = ExecutionRequest::serial(&request);
    let (first, source, tail, old_reader, entity, old_id, new_id) = {
        let (runtime, contract) = fixture();
        let (_, empty) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let mut tx = runtime
            .begin_branch_transaction(&empty, RelationalTransactionIntent::ordinary())
            .unwrap();
        let old_id = tx.admit_content(&execution, FORMAT, &grid()).unwrap();
        tx.push_batch(batch(&contract, old_id), execution.allocation_policy())
            .unwrap();
        let committed = tx.commit(&execution, &runtime).unwrap();
        let entity = committed
            .changed_records
            .iter()
            .find_map(|record| match record {
                RecordRef::Entity(entity) => Some(*entity),
                _ => None,
            })
            .unwrap();
        // Keep the actual admitted historical field reader, not a decoded body or frame.
        let old_reader = committed.publication().envelope.clone();
        let first = capture(&runtime, &execution);
        let covered = first.covered_position().unwrap();
        let (_, selected) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let mut tx = runtime
            .begin_branch_transaction(&selected, RelationalTransactionIntent::ordinary())
            .unwrap();
        let mut changed = grid();
        changed.samples[0] = 5;
        let new_id = tx.admit_content(&execution, FORMAT, &changed).unwrap();
        assert_eq!(
            new_id,
            ContentRefId::from_hex(
                "a3c86eab92c8d73512bfd18e937d9c0271e9888bd2c5cfc859e48307a338da02"
            )
            .unwrap()
        );
        replace(&mut tx, &execution, &contract, entity, new_id);
        tx.commit(&execution, &runtime).unwrap();
        let tail = runtime
            .durability()
            .canonical_commit_artifacts(&execution)
            .map(Result::unwrap)
            .filter(|artifact| artifact.position() > covered)
            .map(|artifact| {
                RelationalCanonicalCommitArtifact::from_untrusted_bytes(
                    artifact.as_bytes().to_vec(),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(tail.len(), 1);
        let source = Arc::new(Source::capture(&runtime, &execution, &first, covered, None));
        // All runtime/basis/outcome handles end here. Only actual old-reader
        // custody and the explicitly supplied physical recovery inputs survive.
        (first, source, tail, old_reader, entity, old_id, new_id)
    };
    let recovered = reopen_with_tail(
        &execution,
        &first,
        first.bytes().to_vec(),
        source.clone(),
        tail,
    );
    assert_current(&recovered, &execution, entity, old_id, new_id);
    for field in ["left", "right"] {
        assert_eq!(
            old_reader
                .content_for_field(&execution, entity, "test.entity", &locator(field))
                .unwrap()
                .bytes(),
            expected::FRAME
        );
    }
    let membership = first
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect::<Vec<_>>();
    let replacement = recovered
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &membership,
                source: Some(source.clone()),
            },
        )
        .unwrap();
    let carried = replacement
        .members()
        .iter()
        .filter(|member| matches!(member, RelationalCheckpointMember::Carried { .. }))
        .count();
    assert!(
        carried > 0,
        "replacement must preserve genuinely cold physical membership"
    );
    eprintln!(
        "native replacement members: carried={carried} new={}",
        replacement.members().len() - carried
    );
    let replacement_source = Arc::new(Source::capture(
        &recovered,
        &execution,
        &replacement,
        replacement.covered_position().unwrap(),
        Some(&source),
    ));
    drop(recovered);
    drop(first);
    drop(source);
    // Recovery now selects only the replacement checkpoint. This proves Native
    // reconstruction, not physical home publication or filesystem collection.
    let reopened = reopen_with_tail(
        &execution,
        &replacement,
        replacement.bytes().to_vec(),
        replacement_source,
        vec![],
    );
    assert_current(&reopened, &execution, entity, old_id, new_id);
    for field in ["left", "right"] {
        assert_eq!(
            old_reader
                .content_for_field(&execution, entity, "test.entity", &locator(field))
                .unwrap()
                .bytes(),
            expected::FRAME
        );
    }
    drop(reopened);
    drop(replacement);
    let held = budget.available_memory_bytes().unwrap();
    assert!(
        held < before,
        "the retained reader still owns original custody"
    );
    drop(old_reader);
    assert_eq!(budget.available_memory_bytes(), Some(before));
    eprintln!(
        "native lifecycle original={before} retained_history={} final={before}",
        before - held
    );
}
fn capture(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> RelationalNativeCheckpoint {
    runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap()
}
fn assert_current(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    entity: EntityId,
    old: ContentRefId,
    current: ContentRefId,
) {
    assert_selected(runtime, execution, entity, current);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis.read_content(execution, old, FORMAT).unwrap_err(),
        RelationalContentDenial::MissingContent
    );
    // Literal specification vector patched at its fixed first sample position;
    // no production encoder, decoder or hash owner supplies these expected bytes.
    let mut changed = expected::FRAME.to_vec();
    changed[43..51].copy_from_slice(&[5, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        basis
            .read_content(execution, current, FORMAT)
            .unwrap()
            .bytes(),
        changed
    );
}
