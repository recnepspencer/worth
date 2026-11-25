//! Owner-backed Priority equality-index residency court across the real
//! `Published (post-upgrade) -> DeferredImage (reopened before read) -> Published (after read)`
//! lifecycle plus negative proof that non-indexed fields never report `"Ready"`.

use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::WorthQueryApplicationRequestExt,
    program_transition::{
        open_directory_with_transition, worth_query_default_build_id,
        WorthQueryProgramTransitionOpenConfig,
    },
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};
use worth_relational::facade::indexes::DerivedIndexGenerationResidency;

use super::driver::request_scope;
use super::journey::seed_v2_tasks;
use super::observe::index_residency_to_state;
use super::schema::{task_priority_field, task_v2_value_field};
use super::v2_program::v2_request;
use super::v3_program::{v3_request, DaymarkV2ToV3};
use super::v3_queries::AllTasksV3Read;

#[test]
fn priority_index_observer_samples_real_published_deferred_and_unindexed_owner_states() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    seed_v2_tasks(&location, build, execution, &[("t1", "a")]);

    let opened = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open_directory_with_transition::<DaymarkV2ToV3>(
                &phase,
                v2_request(location.clone(), build),
                v3_request(location.clone(), build),
                WorthQueryProgramTransitionOpenConfig::default_bounded(),
            )
            .expect("v2 -> v3 upgrade succeeds"),
        )
    })
    .unwrap();
    let v3_home = (*opened).into_home();

    // Immediately after upgrade (BEFORE any read): Published -> "Ready".
    let upgraded_residency = v3_home
        .runtime()
        .observe_equality_index_residency(&execution, task_priority_field())
        .expect("residency observation succeeds");
    assert_eq!(
        upgraded_residency,
        Some(DerivedIndexGenerationResidency::Published)
    );
    assert_eq!(index_residency_to_state(upgraded_residency), "Ready");

    // Non-indexed field (`TaskV2.V2Value`) reports None -> "None", NEVER "Ready".
    let unindexed_residency = v3_home
        .runtime()
        .observe_equality_index_residency(&execution, task_v2_value_field())
        .expect("residency observation succeeds");
    assert_eq!(unindexed_residency, None);
    assert_eq!(index_residency_to_state(unindexed_residency), "None");
    v3_home.close(&execution).unwrap().reclaim();

    // Reopen the real persisted v3 Home from disk and sample BEFORE any read:
    // Native holds the persisted index image lazily as `DeferredImage` ("Cold"),
    // proving the observer does not silently warm or fake "Ready".
    let reopened = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open_directory_with_transition::<DaymarkV2ToV3>(
                &phase,
                v2_request(location.clone(), build),
                v3_request(location, build),
                WorthQueryProgramTransitionOpenConfig::default_bounded(),
            )
            .expect("reopen v3 succeeds"),
        )
    })
    .unwrap();
    let v3_reopened = (*reopened).into_home();
    let before_read = v3_reopened
        .runtime()
        .observe_equality_index_residency(&execution, task_priority_field())
        .expect("residency observation before read succeeds");
    assert_eq!(
        before_read,
        Some(DerivedIndexGenerationResidency::DeferredImage)
    );
    assert_eq!(index_residency_to_state(before_read), "Cold");

    // Executing a read demands the deferred checkpoint image into Published ("Ready").
    let scope = request_scope();
    let runtime = v3_reopened.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let rows = runtime
        .request(&principal, &scope)
        .query(AllTasksV3Read("p-main".to_owned()))
        .execute()
        .expect("v3 read succeeds");
    assert_eq!(rows.rows().len(), 1);
    let after_read = v3_reopened
        .runtime()
        .observe_equality_index_residency(&execution, task_priority_field())
        .expect("residency observation after read succeeds");
    assert_eq!(after_read, Some(DerivedIndexGenerationResidency::Published));
    assert_eq!(index_residency_to_state(after_read), "Ready");

    v3_reopened.close(&execution).unwrap().reclaim();
}
