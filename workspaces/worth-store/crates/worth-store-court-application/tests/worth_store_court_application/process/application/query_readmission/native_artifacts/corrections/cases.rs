//! Cold original/lineage joins through the public Host opener, before service.
use super::*;
use worth_query_execution::facade::primary_graph::QueryHomeCorrectionLineageDenial as Denial;
use worth_query_execution::facade::primary_graph::WorthQueryAftermathCausalityReadDenial as RecordDenial;
#[test]
fn co_published_correction_lineage_reaches_retained_material_gate_after_graceful_reopen() {
    let directory = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    let first = with_bootstrap_advancement_from_request(execution, |phase| {
        let mut source = Source::fresh(directory.path(), &phase);
        let original = source.publish(&execution, source.update(8));
        let reverse = source.correction(
            &execution,
            &rows::Row::reverse(&original, &original, &original, None),
        );
        let first = source.first_fact(&execution);
        let reapply = source.correction(
            &execution,
            &rows::Row::reapply(&reverse, &original, &original, &reverse, &reverse),
        );
        let reverse_again = source.correction(
            &execution,
            &rows::Row::reverse(&reapply, &reapply, &original, Some(&reapply)),
        );
        source.correction(
            &execution,
            &rows::Row::reapply(
                &reverse_again,
                &original,
                &original,
                &reverse,
                &reverse_again,
            ),
        );
        assert_eq!(
            administration::rows(
                source.home.relational().runtime(),
                &execution,
                "worth-query-aftermath-causality"
            )
            .len(),
            4
        );
        source.close(&execution);
        first
    })
    .unwrap();
    let refused = reopen_refusal(directory.path(), execution);
    assert!(
        matches!(correction_denial(&refused), Denial::RetainedMaterialRequired { record: RecordRef::Entity(record) } if *record == first),
        "{:?}",
        correction_denial(&refused)
    );
    refused
        .close(&execution)
        .unwrap_or_else(|refused| panic!("same original Home close: {refused:?}"));
}
#[derive(Clone, Copy, Debug)]
enum Fault {
    WrongOriginalPair,
    WrongParent,
    DoubleConsumption,
    UnassociatedNative,
    RewrittenCanonicalRow,
}
fn refuse_fault(fault: Fault) {
    let directory = tempfile::tempdir().unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = ExecutionRequest::leased(&lease);
    let record = with_bootstrap_advancement_from_request(execution, |phase| {
        let mut source = Source::fresh(directory.path(), &phase);
        let original = source.publish(&execution, source.update(8));
        let second = source.publish(&execution, source.update(9));
        let mut row = rows::Row::reverse(&second, &original, &original, None);
        match fault {
            Fault::WrongOriginalPair => {
                // Both identities are real originals. Pair the second World
                // original with the first Native receipt, including its root.
                row.source.world = second.world.clone();
                row.root.world = second.world.clone();
                source.correction(&execution, &row);
            }
            Fault::WrongParent => {
                row.parent = original.clone();
                source.correction(&execution, &row);
            }
            Fault::UnassociatedNative => {
                let native = source.home.relational().runtime();
                let batch = rows::append(native, &execution, source.update(7), source.count, &row);
                administration::commit(native, &execution, batch);
            }
            Fault::RewrittenCanonicalRow => {
                source.correction(&execution, &row);
                let id = source.first_fact(&execution);
                // A later real write changes the canonical fact. Its fields
                // must still belong to the original created-at receipt.
                administration::update(
                    source.home.relational().runtime(),
                    &execution,
                    id,
                    "aftermath-causality",
                    "outcome-identity",
                    AspectValue::UInt64(source.count as u64),
                );
            }
            Fault::DoubleConsumption => {
                let reversed = source.correction(&execution, &row);
                let reapplied = source.correction(
                    &execution,
                    &rows::Row::reapply(&reversed, &original, &original, &reversed, &reversed),
                );
                source.correction(
                    &execution,
                    &rows::Row::reapply(&reapplied, &original, &original, &reversed, &reversed),
                );
            }
        }
        let rows = administration::rows(
            source.home.relational().runtime(),
            &execution,
            "worth-query-aftermath-causality",
        );
        let record = rows.last().unwrap().entity_id;
        source.close(&execution);
        record
    })
    .unwrap();
    let refused = reopen_refusal(directory.path(), execution);
    match (fault, correction_denial(&refused)) {
        (Fault::WrongOriginalPair, Denial::WorldOriginalMismatch { record: observed })
        | (Fault::DoubleConsumption, Denial::InconsistentRelationships { record: observed })
        | (Fault::UnassociatedNative, Denial::MissingWorldOriginal { record: observed }) => {
            assert_eq!(observed, &RecordRef::Entity(record))
        }
        (
            Fault::WrongParent,
            Denial::Record {
                entity,
                cause: RecordDenial::Unavailable,
                ..
            },
        )
        | (
            Fault::RewrittenCanonicalRow,
            Denial::Record {
                entity,
                cause: RecordDenial::InconsistentCanonicalRelationships,
                ..
            },
        ) => assert_eq!(*entity, record),
        (_, other) => panic!("{fault:?}: exact typed correction refusal expected, got {other:?}"),
    }
    refused
        .close(&execution)
        .unwrap_or_else(|refused| panic!("same original Home close: {refused:?}"));
}

#[test]
fn cold_correction_rows_refuse_a_world_original_paired_with_another_native_receipt() {
    refuse_fault(Fault::WrongOriginalPair);
}

#[test]
fn cold_correction_rows_refuse_a_parent_outside_the_co_committed_child() {
    refuse_fault(Fault::WrongParent);
}

#[test]
fn cold_correction_rows_refuse_consuming_the_same_active_reversal_twice() {
    refuse_fault(Fault::DoubleConsumption);
}

#[test]
fn cold_correction_rows_refuse_native_effects_without_a_world_original() {
    refuse_fault(Fault::UnassociatedNative);
}

#[test]
fn cold_correction_rows_refuse_rewritten_canonical_facts() {
    refuse_fault(Fault::RewrittenCanonicalRow);
}
