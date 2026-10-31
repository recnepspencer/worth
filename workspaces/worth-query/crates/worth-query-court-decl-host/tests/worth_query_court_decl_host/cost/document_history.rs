//! Warm ordinary append; retained-history depth varies, model and local edit do not.
use crate::certification::document_history::recording::{begin, close, finish, ready};
use crate::certification::document_history::{attach, head, value, App, Document};
use std::time::Instant;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryHistoryEditOutcome as Outcome,
    WorthQuerySourceCompletion,
};
use worth_query_host::facade::primary_graph::{
    advancement_observations_on_this_thread_for_test, WorthQueryExecutionObservation,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::publish_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::{DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY},
};

fn filled<'a>(host: &'a App, count: usize) -> Document<'a> {
    let mut document = attach(host, head(host), DOCUMENT_IDENTITY);
    for offset in 0..count {
        let days = if offset % 2 == 0 { 3 } else { 7 };
        document = finish(
            host,
            ready(
                host,
                begin(
                    host,
                    document,
                    DOCUMENT_IDENTITY,
                    days,
                    0x9182_0600 + (count as u64) * 100 + offset as u64,
                ),
            ),
            days,
        )
        .0;
    }
    document
}

#[test]
fn append_probes_only_local_records_as_retained_history_grows() {
    let host = publish_on_first_program();
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    // The actual owning journals retain every preceding step/head. These odd
    // depths finish at3, so both measured operations are exactly the same3->7.
    let short = filled(&host, 1);
    let longer = filled(&host, 15);
    assert_eq!(value(&host, &short), 3);
    assert_eq!(value(&host, &longer), 3);
    let mut samples = Vec::new();
    let mut documents = Vec::new();
    for (depth, mut document) in [(1, short), (15, longer)] {
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let parent = document.current_step(&request).unwrap();
        let native_observer = WorthQueryExecutionObservation::install_on_this_thread();
        let journal_observer = document.observe_history_enrollment_for_test();
        let _ = advancement_observations_on_this_thread_for_test();
        let started = Instant::now();
        let finished = ready(
            &host,
            begin(
                &host,
                document,
                DOCUMENT_IDENTITY,
                7,
                0x9182_7000 + depth as u64,
            ),
        )
        .finish(&request)
        .unwrap();
        let elapsed = started.elapsed();
        let native = advancement_observations_on_this_thread_for_test();
        let journal = journal_observer.snapshot();
        drop(journal_observer);
        drop(native_observer);
        // Qualification and disposal are outside the measured interval.
        let Outcome::Recorded {
            step, completion, ..
        } = finished.outcome
        else {
            panic!("one fresh completed local append")
        };
        let WorthQuerySourceCompletion::Fresh {
            receipt, result, ..
        } = completion
        else {
            panic!("the original ordinary action must perform")
        };
        assert_eq!(result.retention_days, 7);
        assert_eq!(step.parent(), Some(parent.id()));
        assert_eq!(
            step.endpoint().selected_commit(),
            receipt.committed_product_publication().composite_commit()
        );
        drop(receipt);
        assert_eq!(value(&host, &finished.document), 7);
        let work: u64 = native.iter().map(|entry| entry.report.charged_work()).sum();
        let peak = native
            .iter()
            .map(|entry| entry.report.physical().peak_charged_memory_bytes())
            .max()
            .unwrap();
        assert!(native.iter().all(|entry| entry.refusal.is_none()));
        assert!(work > 0);
        eprintln!("append depth={depth}; journal={journal:?}; native_work={work}; native_peak_charged={peak}; elapsed={elapsed:?}");
        samples.push((journal, work, peak));
        documents.push((depth, finished.document));
    }
    let (short, short_work, _) = samples[0];
    let (longer, longer_work, _) = samples[1];
    for (small, large) in [
        (short.node_probes, longer.node_probes),
        (short.attempt_probes, longer.attempt_probes),
        (short.custody_probes, longer.custody_probes),
        (short.child_index_probes, longer.child_index_probes),
        (short.selection_accesses, longer.selection_accesses),
    ] {
        assert!(
            small > 0,
            "the named actual storage boundary must be observed"
        );
        assert_eq!(
            large, small,
            "a local append must not inspect old step records"
        );
    }
    // std's balanced index paths and their original conservative admission
    // quotes can grow with depth. A 15x history must stay inside a coarse
    // sublinear envelope; these are neither physical-byte nor CPU-time claims.
    assert!(short.requested_backing_bytes > 0);
    assert!(longer.requested_backing_bytes <= short.requested_backing_bytes * 3);
    assert!(short.ticket_backing_bytes <= short.requested_backing_bytes);
    assert!(longer.ticket_backing_bytes <= longer.requested_backing_bytes);
    assert!(
        longer_work <= short_work * 3,
        "receiving-owner validation must not replay the history"
    );
    assert_eq!(value(&host, &untouched), 7);
    for (depth, document) in documents {
        close(&host, document, depth + 2);
    }
    close(&host, untouched, 1);
}
