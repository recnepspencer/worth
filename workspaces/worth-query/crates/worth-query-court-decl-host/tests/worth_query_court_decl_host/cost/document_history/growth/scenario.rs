//! Real authored chains and sibling futures, qualified independently of measurements.
use super::{
    host,
    measurement::{Cost, Journal, Meter},
};
use crate::certification::document_history::recording::{begin, ready};
use crate::certification::document_history::{assert_page_entry, attach, head, value};
use std::time::Instant;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryHistoryEditOutcome as Outcome,
    WorthQueryHistoryRedoChoice, WorthQuerySourceCompletion,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    operator_identity::{authenticate_operator, request_scope},
    retention_entry::{DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY},
};

#[derive(Debug)]
pub(super) struct Sample {
    pub edit: Cost,
    pub distant_selection: Cost,
    pub return_selection: Cost,
    pub historical_read: Cost,
    pub diagnostics: Cost,
}

pub(super) fn run(depth: usize, width: usize) -> Sample {
    assert!(depth > 0 && width >= 2);
    let setup = Instant::now();
    let app = host::install(depth + width + 8);
    let untouched = attach(&app, head(&app), RELATED_DOCUMENT_IDENTITY);
    let mut document = attach(&app, head(&app), DOCUMENT_IDENTITY);
    host::window(&app, &mut document, depth + width + 8);
    let (root, context) = {
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        let request = app.request(&principal, &scope);
        let root = document.current_step(&request).unwrap();
        let context = document
            .history_page(&request, host::prefix(), None)
            .unwrap()
            .context()
            .clone();
        (root, context)
    };
    let journal = document.observe_history_enrollment_for_test();
    let snapshot = || {
        let report = journal.snapshot();
        Journal {
            visits: [
                report.node_probes,
                report.attempt_probes,
                report.custody_probes,
                report.child_index_probes,
                report.selection_accesses,
            ],
            frontier: report.retention_frontier_probes,
            retention_work: report.retention_quoted_work,
            requested_bytes: report.requested_backing_bytes,
            backed_bytes: report.ticket_backing_bytes,
        }
    };
    let mut next_key = 0x9182_d000_0000_u64;
    let mut key = || {
        next_key += 1;
        next_key
    };
    let mut first_three = None;
    for offset in 0..depth {
        // Even depths start2, then3/7/.../3; odd depths start3/7/.../3.
        // Every source changes its value and must return an actual fresh record.
        let days = if depth.is_multiple_of(2) && offset == 0 {
            2
        } else if offset % 2 == depth % 2 {
            7
        } else {
            3
        };
        let (next, step) = host::author(&app, document, days, key());
        document = next;
        if days == 3 && first_three.is_none() {
            first_three = Some(step);
        }
    }
    let distant = first_three.unwrap();
    let anchor = {
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        document
            .current_step(&app.request(&principal, &scope))
            .unwrap()
    };
    assert_eq!(value(&app, &document), 3);
    let mut first_sibling = None;
    let mut setup_maintenance = Cost::default();
    for _ in 0..width {
        let (next, child) = host::author(&app, document, 7, key());
        document = next;
        assert_eq!(child.parent(), Some(anchor.id()));
        if first_sibling.is_none() {
            first_sibling = Some(child);
        }
        let completion = host::select(&app, &mut document, &context, &anchor, key());
        assert_eq!(&anchor, completion.selection().step());
        let meter = Meter::start(snapshot());
        host::maintain(&app, &mut document, &context);
        setup_maintenance.accumulate(meter.finish(snapshot()));
    }
    let setup_construction = setup.elapsed() - setup_maintenance.elapsed;
    // A long setup is not part of the initiating request's lifetime. The
    // measured gesture and delivery use fresh authority under the same policy.
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let request = app.request(&principal, &scope);
    let old = document.selection(&request).unwrap();
    let meter = Meter::start(snapshot());
    document.check_history_command_context(&context).unwrap();
    let finished = ready(&app, begin(&app, document, DOCUMENT_IDENTITY, 7, key()))
        .finish(&request)
        .unwrap();
    let edit = meter.finish(snapshot());
    let Outcome::Recorded {
        step, completion, ..
    } = finished.outcome
    else {
        panic!("the measured source must genuinely record")
    };
    let WorthQuerySourceCompletion::Fresh {
        result, receipt, ..
    } = completion
    else {
        panic!("the measured source must really perform")
    };
    assert_eq!(result.retention_days, 7);
    assert_eq!(step.parent(), Some(anchor.id()));
    assert_eq!(
        step.endpoint().selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    drop(receipt);
    document = finished.document;
    assert_eq!(value(&app, &document), 7);
    assert_ne!(document.selection(&request).unwrap(), old);

    let meter = Meter::start(snapshot());
    let completion = host::select(&app, &mut document, &context, &distant, key());
    let distant_selection = meter.finish(snapshot());
    assert_eq!(&distant, completion.selection().step());
    assert_eq!(
        document.selection(&request).unwrap(),
        *completion.selection()
    );
    assert_ne!(
        completion.selection().physical_endpoint().product_branch(),
        distant.endpoint().product_branch()
    );
    assert_eq!(value(&app, &document), 3);
    let meter = Meter::start(snapshot());
    let completion = host::select(&app, &mut document, &context, &anchor, key());
    let return_selection = meter.finish(snapshot());
    assert_eq!(&anchor, completion.selection().step());

    let sibling = first_sibling.unwrap();
    let meter = Meter::start(snapshot());
    let preview = document
        .commands(&context)
        .unwrap()
        .preview(&request, sibling.id())
        .unwrap();
    let read = preview.read(&document, &request).unwrap();
    let historical_read = meter.finish(snapshot());
    assert_eq!(&sibling, preview.step());
    assert_eq!(read.source(), sibling.endpoint());
    assert_eq!(read.rows()[0].retention_days, 7);
    assert_eq!(value(&app, &document), 3);
    let started = Instant::now();
    drop(read);
    preview.close();
    let historical_holder_release = started.elapsed();

    let meter = Meter::start(snapshot());
    let page = document
        .history_page(&request, host::prefix(), None)
        .unwrap();
    let redo = document
        .redo_choices(&request, host::prefix(), None)
        .unwrap();
    let diagnostics = meter.finish(snapshot());
    assert_eq!(page.examined_entries(), 1);
    assert_eq!(page.rows().len(), 1);
    assert_page_entry(page.rows()[0].step(), &root);
    assert_eq!(redo.choice(), WorthQueryHistoryRedoChoice::ChoiceRequired);
    assert_eq!(redo.children().examined_entries(), 1);
    assert_eq!(redo.children().rows().len(), 1);
    assert_page_entry(redo.children().rows()[0].step(), &sibling);
    assert!(redo.children().continuation().is_some());
    drop((page, redo));
    assert_eq!(value(&app, &untouched), 7);
    let meter = Meter::start(snapshot());
    host::maintain(&app, &mut document, &context);
    let maintenance = meter.finish(snapshot());
    let meter = Meter::start(snapshot());
    let closed = app
        .with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    let untouched_closed = app
        .with_host_advancement(|phase| untouched.close(&phase).unwrap())
        .unwrap();
    let document_close = meter.finish(snapshot());
    // Initial head, depth edits, width sibling edits and width return checkouts,
    // then the measured edit and its two genuine selections must all discharge.
    assert_eq!(closed.closed_heads(), depth + 2 * width + 4);
    assert_eq!(untouched_closed.closed_heads(), 1);
    let closed_heads = (closed.closed_heads(), untouched_closed.closed_heads());
    assert!(app.branches().pending_cleanup().is_empty());
    drop(journal);
    drop(principal);
    let started = Instant::now();
    drop(app);
    let runtime_drop = started.elapsed();
    let sample = Sample {
        edit,
        distant_selection,
        return_selection,
        historical_read,
        diagnostics,
    };
    eprintln!("history depth={depth} width={width}; repetitions=1; variance=unestimated; timing=telemetry_only; setup_construction={setup_construction:?}; setup_maintenance={setup_maintenance:?}; phases={sample:?}; historical_read_and_preview_release={historical_holder_release:?}; maintenance={maintenance:?}; document_close={document_close:?}; closed_heads={closed_heads:?}; runtime_drop={runtime_drop:?}");
    sample
}
