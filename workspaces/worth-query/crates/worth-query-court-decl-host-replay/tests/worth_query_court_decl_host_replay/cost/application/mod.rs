mod history;
mod installation;
use super::observation::*;
use model::{
    document_history::RetentionHistory,
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP0,
};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_query_host::facade::application_entry::*;
use worth_query_replay::facade::{
    WorthQueryCertificationApplicationWork, WorthQueryCertificationCostRuntimeExt,
};
use worth_query_testkit_document_retention::facade::document_retention_model as model;
type Schema = model::schema::DocumentRetentionSchema;
type App = model::host::DocumentRetentionRuntime<RetentionProgramP0>;
type Document<'a> = WorthQueryApplicationHistoryDocument<'a, Schema, RetentionHistory>;

fn measure(d: Dimensions) -> Sample {
    let app = installation::install(d);
    let mut document = history::attach(&app);
    for step in 0..d.history {
        let (next, recorded, _) = history::edit(
            &app,
            document,
            if step % 2 == 0 { 3 } else { 7 },
            1000 + step as u64,
        );
        document = next;
        if step % 16 == 15 {
            history::undo(&app, &document, 30000 + step as u128 * 2);
            history::redo(&app, &document, recorded.id(), 30001 + step as u128 * 2);
        }
        if step % 8 == 7 {
            history::unrelated(&app, step);
        }
    }
    let host = super::native::request();
    let execution = ExecutionRequest::serial(&host);
    let branch = {
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        document
            .selection(&app.request(&principal, &scope))
            .unwrap()
            .physical_endpoint()
            .product_branch()
    };
    let before = app
        .with_host_advancement(|phase| {
            app.capture_certification_cost_scope(&phase, branch)
                .unwrap()
        })
        .unwrap();
    let ((document, step, touched), report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            history::edit(&app, document, 5, 9999)
        })
        .unwrap();
    let after = app
        .with_host_advancement(|phase| app.observe_certification_cost(&phase, &before).unwrap())
        .unwrap();
    let mut sample = Sample::new(d);
    sample.record(Phase::HistoryRecord, "charged_work", report.charged_work());
    record(&mut sample, after.application_work());
    sample.commits = (after.world_history_after().installed_commits()
        - after.world_history_before().installed_commits()) as u64;
    sample.touched.push(touched);
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let request = app.request(&principal, &scope);
    let (read, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            document.read(&request).unwrap()
        })
        .unwrap();
    assert_eq!(read.rows()[0].retention_days, 5);
    sample.record(
        Phase::ReadAdmission,
        "document_read_work",
        report.charged_work(),
    );
    drop(read);
    let (_, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            let page = document
                .history_page(
                    &request,
                    WorthQueryHistoryPageLimits {
                        maximum_entries: std::num::NonZeroUsize::MIN,
                        maximum_examined_entries: std::num::NonZeroUsize::MIN,
                        maximum_owned_bytes: std::num::NonZeroUsize::new(16384).unwrap(),
                    },
                    None,
                )
                .unwrap();
            assert!(!page.rows().is_empty());
        })
        .unwrap();
    sample.record(
        Phase::ReadAdmission,
        "history_page_work",
        report.charged_work(),
    );
    let (_, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            history::undo(&app, &document, 10000)
        })
        .unwrap();
    sample.record(Phase::Navigation, "undo_work", report.charged_work());
    assert_ne!(document.current_step(&request).unwrap().id(), step.id());
    app.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
    sample
}

fn record(sample: &mut Sample, work: WorthQueryCertificationApplicationWork) {
    sample.record(
        Phase::CommitPreparation,
        "provider_preparations",
        work.provider_session_preparations(),
    );
    sample.record(
        Phase::ReadAdmission,
        "provider_readmissions",
        work.provider_session_readmissions(),
    );
    sample.record(
        Phase::Invariants,
        "state_loads",
        work.invariant_state_loads(),
    );
    sample.record(Phase::Invariants, "executions", work.invariant_executions());
    sample.record(
        Phase::Publication,
        "prepared_commits",
        work.prepared_commits(),
    );
    sample.record(
        Phase::Rebase,
        "retained_resolutions",
        work.retained_resolutions(),
    );
}

#[test]
fn public_document_edit_read_page_and_undo_expose_growth() {
    let samples = (Dimensions {
        records: 128,
        history: 64,
        installed: 2,
    })
    .points()
    .map(measure);
    for sample in &samples {
        eprintln!("APPLICATION_SCALE {sample:?}");
    }
    super::ratchet::verify("application", &samples);
}
