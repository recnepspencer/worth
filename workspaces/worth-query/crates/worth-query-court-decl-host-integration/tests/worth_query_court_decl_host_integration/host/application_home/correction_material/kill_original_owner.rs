//! C3.6 real process-kill court: an original owner held across a true OS kill
//! (without handoff_for_reopen or graceful close) readmits under fresh
//! authenticated authority, survives a dependency edit, and advances/closes
//! matching the uninterrupted reference while guarding StaleIncarnation and
//! OwnershipMaterialUnavailable.
use super::*;
use crate::process::host::durable_retry::child_reexecution::{
    drive_test, expected, location, wait_for_kill,
};
use std::num::NonZeroU64;
use worth_foundational::facade::{HomeIdentity, ProgramSupportObligationIdentity, ValueDecodeBounds};
use worth_query_host::facade::{
    application_entry::{WorthQueryOriginalOperation, WorthQuerySourceOnlyAdvance},
    primary_graph::WorthQueryOriginalOperationDenial as Denial,
};

const TEST_PATH: &str = "host::application_home::correction_material::kill_original_owner::original_owner_held_across_kill_advances_after_dependency_edit_matching_reference";

#[derive(serde::Serialize, serde::Deserialize)]
struct HeldOwnerRecord {
    home_bytes: [u8; 16],
    acquire_position: u64,
    root_entity: (u32, u64, u32),
    entity_count: usize,
    adjacency_count: usize,
}

fn entity_parts(id: worth_relational::facade::identity::EntityId) -> (u32, u64, u32) {
    (id.partition_value(), id.local_slot_value(), id.generation_value())
}

fn uninterrupted_reference(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> (String, (u32, u64, u32), usize, usize) {
    let path = fixture::location();
    let home = open(path, execution);
    let result = {
        let sc = scope();
        let principal = label::authentication::authenticate_external(
            home.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &sc,
        );
        let req = home.runtime().request(&principal, &sc);
        let description = req
            .mutate(LabelEdit {
                status: "open".into(),
                label: "retained".into(),
            })
            .without_source()
            .idempotency_owned("original-owner-kill".to_owned())
            .unwrap_or_else(|(_, _, cause)| panic!("reference capture: {cause:?}"));
        let operation = description
            .retain_durable_operation::<program::HomeLabelProgram, program::LabelAction>(
                &req,
                home.runtime(),
            )
            .unwrap_or_else(|(_, cause)| panic!("reference retain: {cause:?}"));
        drop(edit(&home));
        let completed = match operation.advance(&req) {
            WorthQuerySourceOnlyAdvance::Completed(c) => c,
            other => panic!("reference advance must complete: {:?}", std::mem::discriminant(&other)),
        };
        assert_eq!(completed.original_input().label, "retained");
        completed
            .close(&req)
            .unwrap_or_else(|(_, cause)| panic!("reference close op: {cause:?}"));
        let rows = req.query(label::AccountSummaryParameters).execute().unwrap();
        let source = &rows.observed_sources()[0];
        (
            rows.rows()[0].label.clone(),
            entity_parts(source.root_entity_for_test()),
            source.footprint_for_test().entities_for_test().len(),
            source.footprint_for_test().adjacencies.len(),
        )
    };
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("reference home close: {cause:?}"))
        .reclaim();
    result
}

#[test]
fn original_owner_held_across_kill_advances_after_dependency_edit_matching_reference() {
    drive_test(TEST_PATH, writer, reader);
}

fn writer() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(location(), execution);
    let sc = scope();
    let principal = label::authentication::authenticate_external(
        home.runtime().installed_schema(),
        "alice",
        Duration::from_secs(120),
        &sc,
    );
    let req = home.runtime().request(&principal, &sc);
    let before = req.query(label::AccountSummaryParameters).execute().unwrap();
    let source = &before.observed_sources()[0];
    let root_entity = entity_parts(source.root_entity_for_test());
    let entity_count = source.footprint_for_test().entities_for_test().len();
    let adjacency_count = source.footprint_for_test().adjacencies.len();
    drop(before);

    let description = req
        .mutate(LabelEdit {
            status: "open".into(),
            label: "retained".into(),
        })
        .without_source()
        .idempotency_owned("original-owner-kill".to_owned())
        .unwrap_or_else(|(_, _, cause)| panic!("capture original: {cause:?}"));
    let operation = description
        .retain_durable_operation::<program::HomeLabelProgram, program::LabelAction>(
            &req,
            home.runtime(),
        )
        .unwrap_or_else(|(_, cause)| panic!("retain durable original: {cause:?}"));
    let identity = operation
        .original_identity()
        .expect("Directory Home original must record its durable obligation identity");
    let record = HeldOwnerRecord {
        home_bytes: identity.home().bytes(),
        acquire_position: identity.acquire_position().get(),
        root_entity,
        entity_count,
        adjacency_count,
    };
    std::fs::write(expected(), serde_json::to_vec(&record).unwrap()).unwrap();
    // Hold the LIVE `operation` and `home` across OS kill: NO handoff_for_reopen, NO close!
    let _held = (&operation, &home);
    wait_for_kill();
}

fn reader() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let reference = uninterrupted_reference(execution);
    let record: HeldOwnerRecord = serde_json::from_slice(&std::fs::read(expected()).unwrap()).unwrap();
    let identity = ProgramSupportObligationIdentity::from_recorded(
        HomeIdentity::from_observed_bytes(record.home_bytes).unwrap(),
        NonZeroU64::new(record.acquire_position).unwrap(),
    );
    let bounds = ValueDecodeBounds::new(8192, 1024, 128, 32);

    // Reopen directly after OS kill without graceful close or handoff_for_reopen:
    // readmit, run dependency edit, advance and close the killed original directly.
    let home = open(location(), execution);
    let (stale_handle, second_identity) = {
        let sc = scope();
        let principal = label::authentication::authenticate_external(
            home.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &sc,
        );
        let req = home.runtime().request(&principal, &sc);
        let before = req.query(label::AccountSummaryParameters).execute().unwrap();
        assert_eq!(before.rows()[0].label, "primary");
        let before_source = &before.observed_sources()[0];
        let before_ownership = (
            before_source.root_entity_for_test(),
            before_source.footprint_for_test().entities_for_test().clone(),
            before_source.footprint_for_test().adjacencies.clone(),
        );
        drop(before);

        let desc = WorthQueryOriginalOperation::readmit_native::<
            _,
            LabelEdit,
            program::HomeLabelProgram,
            program::LabelAction,
        >(identity, &req, home.runtime(), bounds)
        .unwrap_or_else(|cause| panic!("readmit killed original: {cause:?}"));
        let op = desc
            .retain_operation::<program::HomeLabelProgram, program::LabelAction>(home.runtime())
            .unwrap_or_else(|(_, cause)| panic!("retain readmitted original: {cause:?}"));
        drop(edit(&home));
        let completed = match op.advance(&req) {
            WorthQuerySourceOnlyAdvance::Completed(c) => c,
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
                panic!("recovered original advancement refused: {denial:?}")
            }
            _ => panic!("recovered original must complete"),
        };
        assert_eq!(completed.original_input().label, "retained");
        completed
            .close(&req)
            .unwrap_or_else(|(_, cause)| panic!("close recovered original: {cause:?}"));
        let rows = req.query(label::AccountSummaryParameters).execute().unwrap();
        let source = &rows.observed_sources()[0];
        let actual = (
            rows.rows()[0].label.clone(),
            entity_parts(source.root_entity_for_test()),
            source.footprint_for_test().entities_for_test().len(),
            source.footprint_for_test().adjacencies.len(),
        );
        assert_eq!(actual, reference, "killed original owner matches uninterrupted reference");
        assert_eq!(actual.1, record.root_entity, "aggregate root identity stays continuous");
        assert_eq!(actual.2, record.entity_count);
        assert_eq!(actual.3, record.adjacency_count);
        assert_eq!(
            (
                source.root_entity_for_test(),
                source.footprint_for_test().entities_for_test().clone(),
                source.footprint_for_test().adjacencies.clone(),
            ),
            before_ownership,
            "dependency edit and recovered original advance preserve exact entities and adjacencies"
        );
        // Discharged original refuses subsequent readmission as OwnershipMaterialUnavailable.
        assert!(matches!(
            WorthQueryOriginalOperation::readmit_native::<
                _,
                LabelEdit,
                program::HomeLabelProgram,
                program::LabelAction,
            >(identity, &req, home.runtime(), bounds),
            Err(Denial::OwnershipMaterialUnavailable)
        ));

        // Guard StaleIncarnation across reopen with a handed-off original from this incarnation.
        let second = req
            .mutate(LabelEdit {
                status: "open".into(),
                label: "post-kill-followup".into(),
            })
            .without_source()
            .idempotency_owned("original-owner-kill-stale-check".to_owned())
            .unwrap_or_else(|(_, _, cause)| panic!("second capture: {cause:?}"))
            .retain_durable_operation::<program::HomeLabelProgram, program::LabelAction>(
                &req,
                home.runtime(),
            )
            .unwrap_or_else(|(_, cause)| panic!("second retain: {cause:?}"));
        let second_id = second.original_identity().unwrap();
        let (handle, _) = second
            .handoff_for_reopen(&req)
            .unwrap_or_else(|(_, cause)| panic!("handoff second original: {cause:?}"));
        (handle, second_id)
    };
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("post-kill close: {cause:?}"))
        .reclaim();

    let home2 = open(location(), execution);
    {
        let sc = scope();
        let principal = label::authentication::authenticate_external(
            home2.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &sc,
        );
        let req = home2.runtime().request(&principal, &sc);
        assert!(matches!(
            stale_handle.admit(&req),
            Err(Denial::StaleIncarnation)
        ));
        assert!(matches!(
            WorthQueryOriginalOperation::readmit_native::<
                _,
                LabelEdit,
                program::HomeLabelProgram,
                program::LabelAction,
            >(identity, &req, home2.runtime(), bounds),
            Err(Denial::OwnershipMaterialUnavailable)
        ));
        let desc = WorthQueryOriginalOperation::readmit_native::<
            _,
            LabelEdit,
            program::HomeLabelProgram,
            program::LabelAction,
        >(second_identity, &req, home2.runtime(), bounds)
        .unwrap_or_else(|cause| panic!("readmit second original: {cause:?}"));
        let op = desc
            .retain_operation::<program::HomeLabelProgram, program::LabelAction>(home2.runtime())
            .unwrap_or_else(|(_, cause)| panic!("retain second original: {cause:?}"));
        let WorthQuerySourceOnlyAdvance::Completed(completed) = op.advance(&req) else {
            panic!("second original must complete");
        };
        completed
            .close(&req)
            .unwrap_or_else(|(_, cause)| panic!("close second original: {cause:?}"));
    }
    home2
        .close(&execution)
        .unwrap_or_else(|cause| panic!("final close: {cause:?}"))
        .reclaim();
}
