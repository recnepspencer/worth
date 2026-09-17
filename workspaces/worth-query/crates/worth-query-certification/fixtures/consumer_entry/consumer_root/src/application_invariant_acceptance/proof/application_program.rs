use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_topology_entry::{PlanarRead, PlanarSourceAdjustment};

use super::super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

mod custody;
mod dependent_recovery;
pub(super) mod lifecycle;
mod program_contract;
mod readiness_recovery;
mod recovery;
mod settlement;

pub(super) fn performed_source_settles_required_output(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    settlement::performed_source_settles_required_output(foreign);
}

pub(super) fn caller_disposal_before_progress_recovers(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    recovery::caller_disposal_before_progress_recovers(foreign);
    recovery::changed_root_cannot_adopt_stale_prepared_source(foreign);
    dependent_recovery::caller_disposal_after_root_recovers_dependent(foreign);
}

pub(super) fn resource_denial_preserves_source_and_delivery(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    recovery::resource_denial_preserves_source_and_delivery(foreign);
}

pub(super) fn lifecycle_proofs(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    custody::abandoned_and_superseded_preparations_are_bounded(foreign);
    custody::close_before_required_output_start_is_typed(foreign);
    lifecycle::supersession_retires_pending_predecessor(foreign);
    lifecycle::duplicate_retry_does_not_schedule_again(foreign);
    lifecycle::two_forks_preserve_predecessor_output(foreign);
    lifecycle::branch_close_wakes_live_required_output(foreign);
    readiness_recovery::readiness_failure_recovers_exact_pending_output(foreign);
    readiness_recovery::preserved_noop_output_completes_readiness_without_a_signal_successor(
        foreign,
    );
    readiness_recovery::program_demand_closes_and_rejects_a_foreign_request(foreign);
    readiness_recovery::denied_program_producer_releases_the_shared_claim(foreign);
}

pub(super) fn foreign_program_is_denied_before_publication(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let source_world = installation::install(foreign);
    let other_world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(source_world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the source application authenticates its principal");
    let request = source_world.application.request(&principal, &scope);
    let before = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable");
    let source = before.observed_sources()[0].clone();
    let denial = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_004)
        .execute_performed(&other_world.application);
    let Err(denial) = denial else {
        panic!("foreign program meaning must be denied before source publication")
    };
    assert!(format!("{denial:?}").contains("ForeignProgram"));
    let after = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source remains readable after denial");
    assert_eq!(after.rows()[0].y, length(1));
}

pub(super) fn ordinary_source_publication_cannot_bypass_program(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let before = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable");
    let source = before.observed_sources()[0].clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_005)
        .execute();
    let Err(denial) = outcome else {
        panic!("the migrated source operation must require its installed program")
    };
    assert_eq!(
        denial.kind(),
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::ApplicationProgramRequired,
    );
    let after = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the denied source remains readable");
    assert_eq!(after.rows()[0].y, length(1));
}
