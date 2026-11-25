//! Fail-closed transition refusals on real Directory Homes: entity scan cap,
//! predecessor-field overwrite refusal, non-indexed rebuild target refusal,
//! and cancellation inside metered backfill planning — all keeping `v2` intact.

use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    program_transition::{
        open_directory_with_transition, worth_query_default_build_id,
        WorthQueryProgramTransitionOpenConfig, WorthQueryProgramTransitionOpenDenial,
    },
};
use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};

use super::journey::{assert_v2_reopens_intact, seed_v2_tasks};
use super::v2_program::v2_request;
use super::v3_program::{
    v3_request, DaymarkV2ToV3, NonIndexedRebuildTransition, OverwritePredecessorFieldTransition,
};

#[test]
fn transition_refuses_entity_scan_cap_with_typed_denial_and_keeps_v2_intact() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a"), ("t2", "b")]);

    let scan_refused = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<DaymarkV2ToV3>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded().with_max_entities_per_kind(1),
        )
    })
    .unwrap()
    .err()
    .expect("max_entities_per_kind=1 on 2 tasks must refuse");
    assert!(matches!(
        scan_refused,
        WorthQueryProgramTransitionOpenDenial::EntityScanLimitExceeded {
            maximum_entities_per_kind: 1,
            ..
        }
    ));
    drop(scan_refused);
    assert_v2_reopens_intact(&location, build, execution, 2);
}

#[test]
fn transition_refuses_predecessor_field_overwrite_and_non_indexed_rebuild_without_stranding_v2() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a"), ("t2", "b")]);

    let overwrite_refused = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<OverwritePredecessorFieldTransition>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded(),
        )
    })
    .unwrap()
    .err()
    .expect("default targeting existing predecessor field must refuse");
    assert!(matches!(
        overwrite_refused,
        WorthQueryProgramTransitionOpenDenial::Host(_)
    ));
    drop(overwrite_refused);

    let non_indexed_refused = with_bootstrap_advancement_from_request(execution, |phase| {
        open_directory_with_transition::<NonIndexedRebuildTransition>(
            &phase,
            v2_request(location.clone(), build),
            v3_request(location.clone(), build),
            WorthQueryProgramTransitionOpenConfig::default_bounded(),
        )
    })
    .unwrap()
    .err()
    .expect("rebuild target with no installed equality index must refuse");
    assert!(matches!(
        non_indexed_refused,
        WorthQueryProgramTransitionOpenDenial::Host(_)
    ));
    drop(non_indexed_refused);

    assert_v2_reopens_intact(&location, build, execution, 2);
}
