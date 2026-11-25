//! Cold native route readmission authenticates the real prior disposition.
use super::{
    super::super::checkpoint_wire_fixture::{array, edit_original, field, replace_field, rows},
    super::{
        fixture::{basis, capture},
        publication_fixture,
    },
    fixture::*,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{history::CommitId, runtime::RelationalRuntime};

#[derive(Deserialize)]
enum RecordedPublication {
    BirthAfterCodecRetirement { retirement: CommitId },
}

pub(super) fn frames(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> BTreeMap<CommitId, Vec<u8>> {
    runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .map(|artifact| {
            let artifact = artifact.unwrap();
            (artifact.commit_id(), artifact.as_bytes().to_vec())
        })
        .collect()
}

pub(super) fn cold_retired_route(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: &RelationalRuntime,
) {
    let saved = capture(execution, target);
    let (mut cold, receipt) = installed(2);
    cold.durability_recovery()
        .restore_native_checkpoint(execution, &saved, worth_testkit_relational::cold_recovery::members_source(&saved))
        .unwrap();
    let receipt = cold
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (outputs, relations) = produce(execution, &cold, &receipt, 2);
    super::super::fixture::build_index(execution, &cold);
    assert_order(execution, &cold, &outputs, &relations);
    assert_values(
        execution,
        &cold,
        &outputs,
        [
            "codec2:nondefault-0",
            "codec2:nondefault-1",
            "codec2:nondefault-2",
        ],
    );
    let current = basis(&cold).observation().commit_id().unwrap();
    let publications = frames(execution, &cold);
    let regenerated = &publications[&current][66..];
    let origin = field(field(regenerated, "canonical"), "required_output_origin");
    let publication = field(origin, "publication");
    let RecordedPublication::BirthAfterCodecRetirement { retirement } =
        rmp_serde::from_slice(publication).expect("canonical route is born again after retirement");
    let retired_frames = frames(execution, target);
    let foreign = retired_frames
        .iter()
        .find_map(|(id, frame)| {
            let body = field(&frame[66..], "canonical");
            (field(body, "required_output_origin") == [0xc0]
                && rows(field(body, "required_output_codec_retirements")).is_empty())
            .then_some(*id)
        })
        .unwrap();
    for incomplete in [false, true] {
        let untrusted = edit_original(&saved, retirement, |body| {
            let mut events = rows(field(body, "required_output_codec_retirements"));
            let event = events[0];
            let amended = if incomplete {
                let mut kinds = rows(field(event, "kinds"));
                kinds.pop().unwrap();
                replace_field(event, "kinds", &array(&kinds))
            } else {
                replace_field(event, "publication", &rmp_serde::to_vec(&foreign).unwrap())
            };
            events[0] = &amended;
            replace_field(body, "required_output_codec_retirements", &array(&events))
        });
        let (mut rejected, receipt) = installed(2);
        rejected
            .durability_recovery()
            .restore_native_checkpoint(execution, &untrusted, worth_testkit_relational::cold_recovery::members_source(&untrusted))
            .unwrap();
        let receipt = rejected
            .readmit_recovered_initial_schema_installation(receipt)
            .unwrap();
        let (_, installed) = receipt.issue_application_publication_owner();
        let before = capture(execution, &rejected);
        let denied = rejected
            .readmit_required_output_route_absence(
                execution,
                &installed,
                &basis(&rejected),
                &publication_fixture::meaning("native.closed.producer"),
            )
            .unwrap_err();
        if incomplete {
            assert!(
                format!("{denied:?}").contains("actual protected program contract"),
                "{denied:?}"
            );
        } else {
            let worth_relational::facade::transactions::TransactionCommitError::Publication {
                error,
                ..
            } = &denied
            else {
                panic!("foreign publication must fail canonical route validation: {denied:?}");
            };
            assert_eq!(
                error.stage,
                worth_relational::facade::publication::PublicationStage::BundleAssembly
            );
            assert_eq!(error.detail, "exact route has no canonical publication");
        }
        assert_eq!(capture(execution, &rejected), before);
    }
}
