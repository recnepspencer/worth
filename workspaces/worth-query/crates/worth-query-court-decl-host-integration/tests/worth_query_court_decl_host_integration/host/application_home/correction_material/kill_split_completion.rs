//! C3.6 real process-kill court: authored correction durable, derived settlement
//! pending across OS kill, recovered and settled once matching the uninterrupted
//! reference, and stable across a second reopen.
use super::numeric_home::{
    self, model, open_output_home, read_calc, read_rows, DirectoryOutputProgram as OutputProgram,
};
use super::*;
use crate::process::host::durable_retry::child_reexecution::{
    drive_test, expected, location, wait_for_kill,
};
use model::{
    authentication::*,
    calculation,
    correction::RecordedEdit,
    edit::{Edit, Value},
    other_edit::OtherEdit,
    output_program::Root,
    query::Row,
};
use worth_query_host::facade::{
    application_entry::*,
    primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};

const TEST_PATH: &str = "host::application_home::correction_material::kill_split_completion::split_completion_kill_recovers_authored_correction_and_settles_once_matching_reference";
const ORIG_KEY: &str = "split-source-edit";
const OTHER_KEY: &str = "split-disjoint-edit";
const REV_KEY: &str = "split-reverse-with-required-output";

#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
struct WriterRecord {
    original_commit_id: u64,
    original_world_ordinal: u64,
    correction_commit_id: u64,
    correction_world_ordinal: u64,
    pre_settlement_calc: u64,
}

#[derive(Debug, Eq, PartialEq)]
struct SplitReference {
    original_commit_id: u64,
    original_world_ordinal: u64,
    correction_commit_id: u64,
    correction_world_ordinal: u64,
    settled_rows_primary: u64,
    settled_rows_secondary: u64,
    settled_calculation: u64,
    settled_world_ordinal: u64,
}

fn publish_and_settle_source(
    home: &numeric_home::OutputNumericHome,
) -> WorthQueryApplicationCommitReceipt {
    let app = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let req = app.runtime().request(&principal, &scope);

    let orig_key = ORIG_KEY.to_owned();
    let performed = match req
        .advanced()
        .mutate(Edit {
            key: "numeric".to_owned(),
            value: Value::Primary(11),
        })
        .without_source()
        .idempotency(&orig_key)
        .execute_performed::<OutputProgram, Root>(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        WorthQueryApplicationPerformedMutationOutcome::Performed(p) => p,
        _ => panic!("original edit must perform"),
    };
    let original = performed.receipt().clone();
    let mut started = performed
        .start_required_outputs(app, &req, WorthQueryOutputDemandControls::host_policy())
        .unwrap_or_else(|_| panic!("start original required output"));
    assert!(matches!(
        started.required_output_mut().settle(app, &req).unwrap(),
        WorthQueryApplicationProgramOutputProgress::Settled(_)
    ));
    assert_eq!(read_calc(app), 23);

    let other_key = OTHER_KEY.to_owned();
    let other_perf = match req
        .advanced()
        .mutate(OtherEdit {
            key: "numeric".to_owned(),
            value: Value::Secondary(19),
        })
        .without_source()
        .idempotency(&other_key)
        .execute_performed::<OutputProgram, Root>(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        WorthQueryApplicationPerformedMutationOutcome::Performed(p) => p,
        _ => panic!("disjoint edit must perform"),
    };
    let mut other_started = other_perf
        .start_required_outputs(app, &req, WorthQueryOutputDemandControls::host_policy())
        .unwrap_or_else(|_| panic!("start disjoint required output"));
    assert!(matches!(
        other_started
            .required_output_mut()
            .settle(app, &req)
            .unwrap(),
        WorthQueryApplicationProgramOutputProgress::Settled(_)
    ));
    assert_eq!(read_calc(app), 23);
    original
}

fn uninterrupted_reference(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> SplitReference {
    let loc = fixture::location();
    let home = open_output_home("reference", loc.clone(), execution, true);
    let reference = {
        let original = publish_and_settle_source(&home);
        let app = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        let req = app.runtime().request(&principal, &scope);
        let selected = req.corrections().select(&original, numeric_home::limits()).unwrap();
        let WorthQueryRequiredCorrectionPreparation::Prepared(prepared) = req
            .corrections()
            .reverse(&selected)
            .idempotency(REV_KEY)
            .limits(numeric_home::limits())
            .prepare_performed::<RecordedEdit, OutputProgram, Root>(app)
            .unwrap()
        else {
            panic!("reference correction must prepare");
        };
        let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) =
            prepared.publish().unwrap()
        else {
            panic!("reference correction must perform");
        };
        let correction = performed.receipt().clone();
        let ref_causality = correction.aftermath_causality().expect("reference correction lineage");
        assert_eq!(ref_causality.role(), WorthQueryAftermathCausalRole::RecordedReverse);
        assert_eq!(
            ref_causality.recorded_source().relational().commit_id.0,
            original.commit_reference().commit_id.0
        );
        let mut started = performed
            .start_required_outputs(app, &req, WorthQueryOutputDemandControls::host_policy())
            .unwrap_or_else(|_| panic!("reference required output starts"));
        let WorthQueryApplicationProgramOutputProgress::Settled(settled) =
            started.required_output_mut().settle(app, &req).unwrap()
        else {
            panic!("reference output settles");
        };
        assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
        let branch = app.runtime().current_world();
        let rows = read_rows(app, branch);
        let calc = read_calc(app);
        assert_eq!(
            rows,
            [Row {
                operand: None,
                primary: 5,
                secondary: 19
            }]
        );
        assert_eq!(calc, 11);
        SplitReference {
            original_commit_id: original.commit_reference().commit_id.0,
            original_world_ordinal: original
                .committed_product_publication()
                .composite_commit()
                .ordinal(),
            correction_commit_id: correction.commit_reference().commit_id.0,
            correction_world_ordinal: correction
                .committed_product_publication()
                .composite_commit()
                .ordinal(),
            settled_rows_primary: rows[0].primary,
            settled_rows_secondary: rows[0].secondary,
            settled_calculation: calc,
            settled_world_ordinal: numeric_home::head_ordinal(app, branch),
        }
    };
    home.close(&execution)
        .unwrap_or_else(|e| panic!("reference close: {e:?}"))
        .reclaim();
    reference
}

#[test]
fn split_completion_kill_recovers_authored_correction_and_settles_once_matching_reference() {
    drive_test(TEST_PATH, writer, reader);
}

fn writer() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_output_home("writer", location(), execution, true);
    let original = publish_and_settle_source(&home);
    let app = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let req = app.runtime().request(&principal, &scope);
    let selected = req.corrections().select(&original, numeric_home::limits()).unwrap();
    let WorthQueryRequiredCorrectionPreparation::Prepared(prepared) = req
        .corrections()
        .reverse(&selected)
        .idempotency(REV_KEY)
        .limits(numeric_home::limits())
        .prepare_performed::<RecordedEdit, OutputProgram, Root>(app)
        .unwrap()
    else {
        panic!("writer correction must prepare");
    };
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) =
        prepared.publish().unwrap()
    else {
        panic!("writer correction must perform");
    };
    let correction = performed.receipt().clone();
    // Authored correction is durable (primary=5, secondary=19), derived
    // settlement is still pending (calculation=23).
    assert_eq!(
        read_rows(app, app.runtime().current_world()),
        [Row {
            operand: None,
            primary: 5,
            secondary: 19
        }]
    );
    assert_eq!(read_calc(app), 23);
    let record = WriterRecord {
        original_commit_id: original.commit_reference().commit_id.0,
        original_world_ordinal: original.committed_product_publication().composite_commit().ordinal(),
        correction_commit_id: correction.commit_reference().commit_id.0,
        correction_world_ordinal: correction.committed_product_publication().composite_commit().ordinal(),
        pre_settlement_calc: read_calc(app),
    };
    std::fs::write(expected(), serde_json::to_vec(&record).unwrap()).unwrap();
    let _held = (&performed, &selected);
    wait_for_kill();
}

fn reader() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let reference = uninterrupted_reference(execution);
    let recorded: WriterRecord = serde_json::from_slice(&std::fs::read(expected()).unwrap()).unwrap();
    assert_eq!(recorded.original_commit_id, reference.original_commit_id);
    assert_eq!(recorded.original_world_ordinal, reference.original_world_ordinal);
    assert_eq!(recorded.correction_commit_id, reference.correction_commit_id);
    assert_eq!(recorded.correction_world_ordinal, reference.correction_world_ordinal);
    assert_eq!(recorded.pre_settlement_calc, 23);

    let home = open_output_home("first-reopen", location(), execution, false);
    {
        let app = home.runtime();
        let branch = app.runtime().current_world();
        assert_eq!(read_rows(app, branch), [Row { operand: None, primary: 5, secondary: 19 }]);
        assert_eq!(read_calc(app), 23);
        let head_before = numeric_home::head_ordinal(app, branch);
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        let req = app.runtime().request(&principal, &scope);

        let orig_key = ORIG_KEY.to_owned();
        let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::RecoveredCommitted(orig_replay),
        ) = req
            .advanced()
            .mutate(Edit { key: "numeric".to_owned(), value: Value::Primary(11) })
            .without_source()
            .idempotency(&orig_key)
            .execute_performed::<OutputProgram, Root>(app, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
        else {
            panic!("original key must replay RecoveredCommitted after kill");
        };
        assert_eq!(orig_replay.commit_reference().commit_id.0, recorded.original_commit_id);
        let orig_durable = orig_replay.new_product_head().durable().unwrap();
        assert_eq!(orig_durable.selected_commit().ordinal(), recorded.original_world_ordinal);
        let orig_coords = WorthQueryCorrectionSourceCoordinates::new(
            worth_foundational::facade::WorldOwnerCoordinate::Home { owner: orig_durable.branch().owner() },
            orig_durable.selected_commit().ordinal(),
            orig_replay.commit_reference().clone(),
            worth_query_host::facade::primary_graph::WorthQueryCorrectionSourceMember::Primary,
        );
        let selected = req.corrections().select_coordinates(&orig_coords, numeric_home::limits()).unwrap();
        let WorthQueryRequiredCorrectionPreparation::Settled(
            WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
                WorthQueryApplicationMutationOutcome::RecoveredCommitted(replayed),
            ),
        ) = req
            .corrections()
            .reverse(&selected)
            .idempotency(REV_KEY)
            .limits(numeric_home::limits())
            .prepare_performed::<RecordedEdit, OutputProgram, Root>(app)
            .unwrap()
        else {
            panic!("correction retry after kill must answer RecoveredCommitted");
        };
        assert_eq!(replayed.commit_reference().commit_id.0, recorded.correction_commit_id);
        assert_eq!(
            replayed.new_product_head().durable().unwrap().selected_commit().ordinal(),
            recorded.correction_world_ordinal
        );
        let causality = replayed.aftermath_causality().expect("recovered correction lineage");
        assert_eq!(causality.role(), WorthQueryAftermathCausalRole::RecordedReverse);
        assert_eq!(causality.source().relational().commit_id.0, reference.original_commit_id);
        assert_eq!(numeric_home::head_ordinal(app, branch), head_before);
        assert_eq!(read_calc(app), 23);

        let mut handle = req
            .advanced()
            .recover_required_outputs::<OutputProgram, Root>(
                app,
                &replayed,
                calculation::Demand { key: "numeric".to_owned() },
                WorthQueryOutputDemandControls::host_policy(),
            )
            .unwrap_or_else(|e| panic!("recover pending required outputs after kill: {e:?}"));
        let WorthQueryApplicationProgramOutputProgress::Settled(settled) =
            handle.settle(app, &req).unwrap()
        else {
            panic!("recovered required output must settle");
        };
        assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
        assert_eq!(read_calc(app), reference.settled_calculation);
        assert_eq!(numeric_home::head_ordinal(app, branch), reference.settled_world_ordinal);
        assert_eq!(
            read_rows(app, branch),
            [Row {
                operand: None,
                primary: reference.settled_rows_primary,
                secondary: reference.settled_rows_secondary,
            }]
        );
    }
    home.close(&execution).unwrap_or_else(|e| panic!("first reopen close: {e:?}")).reclaim();

    let home2 = open_output_home("second-reopen", location(), execution, false);
    {
        let app = home2.runtime();
        let branch = app.runtime().current_world();
        assert_eq!(
            read_rows(app, branch),
            [Row {
                operand: None,
                primary: reference.settled_rows_primary,
                secondary: reference.settled_rows_secondary,
            }]
        );
        assert_eq!(read_calc(app), reference.settled_calculation);
        assert_eq!(numeric_home::head_ordinal(app, branch), reference.settled_world_ordinal);
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        let req = app.runtime().request(&principal, &scope);
        let mut demand = req
            .demand(calculation::Demand { key: "numeric".to_owned() })
            .start_in_program::<OutputProgram, Root>(app)
            .unwrap();
        let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.settle(&req).unwrap()
        else {
            panic!("second reopen demand settles");
        };
        assert_eq!(settled.producer_contacts_in_this_demand(), 0);
        assert_eq!(read_calc(app), reference.settled_calculation);
    }
    home2.close(&execution).unwrap_or_else(|e| panic!("second reopen close: {e:?}")).reclaim();
}
