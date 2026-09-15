use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryRequiredOutputPreparationDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;

use super::super::super::{authentication, installation};
use super::lifecycle::{controls, perform};
use crate::ConsumerSchema;

pub(super) fn zero_discovery_settles_an_empty_inventory(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let (principal, scope) = principal(&world.application);
    let request = world.application.request(&principal, &scope);
    let mut output = perform(&request, &world.application, "sibling-c", 22, 10_025);
    let settled = loop {
        match output.required_output_mut().advance(&request).unwrap() {
            worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputProgress::Pending => {}
            worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(
        settled
            .output_observations::<
                worth_query_topology_entry::PlanarFinalOutputFeature,
                worth_query_topology_entry::PlanarFinalBodyOutput,
            >()
            .count(),
        0
    );
    assert!(settled.latest_observation().selected_commit().ordinal() > 0);
    assert_eq!(world.application.program_recovery_count_for_test(), 0);
}

pub(super) fn foreign_retired_and_superseded_custody_is_denied(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    foreign_runtime_receipt_is_denied(foreign);
    retired_occurrence_is_closed(foreign);
    older_source_commit_is_superseded(foreign);
}

fn foreign_runtime_receipt_is_denied(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let source_world = installation::install(foreign);
    let target_world = installation::install(foreign);
    let (source_principal, source_scope) = principal(&source_world.application);
    let source_request = source_world
        .application
        .request(&source_principal, &source_scope);
    let output = perform(
        &source_request,
        &source_world.application,
        "anchor-a",
        2,
        10_021,
    );
    let receipt = output.receipt().clone();
    let (target_principal, target_scope) = principal(&target_world.application);
    let target_request = target_world
        .application
        .request(&target_principal, &target_scope);
    assert_demand_denial(
        target_request.recover_required_outputs::<
            crate::ConsumerProgram,
            crate::ConsumerProgramInventory,
            crate::ConsumerProgramRoot,
        >(&target_world.application, &receipt, controls()),
        WorthQueryOutputDemandDenialKind::ForeignSettlement,
    );
}

fn retired_occurrence_is_closed(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let branch = world
        .application
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .unwrap();
    let (principal, scope) = principal(&world.application);
    let request = world
        .application
        .request(&principal, &scope)
        .on_branch(branch);
    let output = perform(&request, &world.application, "anchor-a", 2, 10_022);
    let receipt = output.receipt().clone();
    drop(output);
    drop(request);
    let cleanup = match world.application.on_branch(branch).close() {
        Ok(_) => None,
        Err(
            worth_query_host::facade::primary_graph::WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(
                failure,
            ),
        ) => Some(failure.into_cleanup()),
        Err(denial) => panic!("branch retirement reaches owner cleanup: {denial:?}"),
    };
    let root_request = world.application.request(&principal, &scope);
    assert_demand_denial(
        root_request.recover_required_outputs::<
            crate::ConsumerProgram,
            crate::ConsumerProgramInventory,
            crate::ConsumerProgramRoot,
        >(&world.application, &receipt, controls()),
        WorthQueryOutputDemandDenialKind::Closed,
    );
    assert_eq!(world.application.program_recovery_count_for_test(), 0);
    drop(receipt);
    if let Some(cleanup) = cleanup {
        cleanup.retry().unwrap();
    }
}

fn older_source_commit_is_superseded(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let (principal, scope) = principal(&world.application);
    let request = world.application.request(&principal, &scope);
    let first = perform(&request, &world.application, "anchor-b", 2, 10_023);
    let first_receipt = first.receipt().clone();
    let second = perform(&request, &world.application, "anchor-b", 3, 10_024);
    assert_eq!(world.application.program_recovery_count_for_test(), 1);
    assert_demand_denial(
        request.recover_required_outputs::<
            crate::ConsumerProgram,
            crate::ConsumerProgramInventory,
            crate::ConsumerProgramRoot,
        >(&world.application, &first_receipt, controls()),
        WorthQueryOutputDemandDenialKind::Superseded,
    );
    drop(second);
    drop(first);
}

fn assert_demand_denial<T>(
    result: Result<T, WorthQueryRequiredOutputPreparationDenial>,
    expected: WorthQueryOutputDemandDenialKind,
) {
    let Err(WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial)) = result else {
        panic!("program recovery must return its exact custody denial")
    };
    assert_eq!(denial.kind(), expected);
}

fn principal(
    application: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        ConsumerSchema,
        crate::ConsumerProgram,
    >,
) -> (
    WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema>,
    WorthQueryRequestScope,
) {
    let scope = authentication::request_scope();
    let adapter = authentication::admit(application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the program application authenticates its principal");
    (principal, scope)
}
