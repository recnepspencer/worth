use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{PlanarRead, PlanarSourceAdjustment};

use super::super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

mod custody;
mod dependent_recovery;
mod discovered;
pub(super) mod lifecycle;
mod owner_demand_boundary;
mod program_contract;
mod readiness_recovery;
mod recovery;
mod required_basis;
pub(super) mod root_selection;
mod settlement;

pub(super) fn performed_source_settles_required_output(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    settlement::performed_source_settles_required_output(foreign);
    discovered::performed_source_discovers_required_root(foreign);
    discovered::isolated_source_settles_without_roots(foreign);
    discovered::recovery::newer_discovered_source_retires_recovery(foreign);
    discovered::recovery::foreign_runtime_cannot_recover_discovered_source(foreign);
    discovered::recovery::interrupted_discovery_recovers_both_consumed_roots(foreign);
    discovered::publication_lifecycle::unchanged_roots_join_new_publication(foreign);
    discovered::publication_lifecycle::older_publication_starts_after_newer_root_binding(foreign);
    discovered::publication_lifecycle::running_root_supersession_preserves_sibling(foreign);
    discovered::publication_basis::joined_roots_discover_at_their_own_publication(foreign);
    required_basis::joined_required_root_discovers_at_its_own_publication(foreign);
    discovered::recovery::required_recovery_cannot_claim_discovered_custody(foreign);
    discovered::publication_lifecycle::newer_publication_bounds_abandoned_discovery(foreign);
    secondary_root_settles_independently(foreign);
}

fn secondary_root_settles_independently(
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
    .expect("the multi-root application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let secondary_branch = world
        .application
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the sibling root receives a real product branch");
    let secondary_request = world
        .application
        .request(&principal, &scope)
        .on_branch(secondary_branch);
    let primary_source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the primary root source is readable")
        .observed_sources()[0]
        .clone();
    let primary_outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(primary_source)
        .idempotency(&10_006)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("the explicitly selected primary root reaches publication");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(primary_performed) =
        primary_outcome
    else {
        panic!("the primary root must retain performed delivery")
    };
    let mut primary_started = primary_performed
        .start_required_outputs(
            &request,
            WorthQueryOutputDemandControls::new(
                NonZeroUsize::new(4_096).unwrap(),
                NonZeroUsize::new(8_192).unwrap(),
            ),
        )
        .unwrap_or_else(|failure| panic!("primary root starts: {:?}", failure.denial()));

    let secondary_source = secondary_request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the secondary root source is readable")
        .observed_sources()[0]
        .clone();
    let secondary_outcome = secondary_request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(3),
        })
        .expect_source(secondary_source)
        .idempotency(&10_009)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerSecondaryProgramRoot>(
            &world.application,
        )
        .expect("the explicitly selected secondary root reaches publication");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(secondary_performed) =
        secondary_outcome
    else {
        panic!("the secondary root must retain performed delivery")
    };
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let mut secondary_started = secondary_performed
        .start_required_outputs(&secondary_request, controls)
        .unwrap_or_else(|failure| panic!("secondary root starts: {:?}", failure.denial()));
    loop {
        match secondary_started
            .required_output_mut()
            .advance(&secondary_request)
            .expect("the selected secondary root advances")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(_) => break,
        }
    }
    loop {
        match primary_started
            .required_output_mut()
            .advance(&request)
            .expect("the primary root remains live while its sibling settles")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(_) => break,
        }
    }
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the primary source mutation committed")
            .rows()[0]
            .y,
        length(2)
    );
    assert_eq!(
        secondary_request
            .query(PlanarRead {
                body_key: "anchor-b".to_owned(),
            })
            .execute()
            .expect("the secondary source mutation committed")
            .rows()[0]
            .y,
        length(3)
    );
    drop(secondary_started);
    drop(primary_started);
    world
        .application
        .on_branch(secondary_branch)
        .close()
        .expect("the sibling root branch closes after both roots settle");
}

pub(super) fn caller_disposal_before_progress_recovers(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    recovery::caller_disposal_before_progress_recovers(foreign);
    recovery::snapshot_pressure_preserves_recoverable_source(foreign);
    recovery::superseded_completion_is_terminal(foreign);
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
    owner_demand_boundary::producer_lifecycle_probe(foreign);
    custody::abandoned_and_superseded_preparations_are_bounded(foreign);
    custody::close_before_required_output_start_is_typed(foreign);
    lifecycle::supersession_retires_pending_predecessor(foreign);
    lifecycle::duplicate_retry_does_not_schedule_again(foreign);
    lifecycle::two_forks_preserve_predecessor_output(foreign);
    lifecycle::branch_close_wakes_live_required_output(foreign);
    readiness_recovery::readiness_failure_recovers_exact_pending_output(foreign);
    readiness_recovery::ready_read_capacity_preserves_completion(foreign);
    readiness_recovery::already_committed_replace_reuses_readiness(foreign);
    readiness_recovery::complete_dependency_aba_advances_the_live_demand(foreign);
    readiness_recovery::published_outputs_hold_no_hidden_read_lease(foreign);
    readiness_recovery::readiness_snapshot_pressure_keeps_published_output_recoverable(foreign);
    readiness_recovery::preserved_noop_output_completes_readiness_without_a_signal_successor(
        foreign,
    );
    readiness_recovery::program_demand_rejects_a_foreign_request_and_releases_on_drop(foreign);
    readiness_recovery::denied_program_producer_releases_the_shared_claim(foreign);
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
