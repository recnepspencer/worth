mod branch;
use branch::fork;

use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandProgress,
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
    WorthQueryProgramOutputCurrentnessDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;
use worth_query_topology_entry::{
    PlanarOutputDemand, PlanarOutputRead, PlanarRead, PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

#[path = "lifecycle/supersession_retires_pending_predecessor.rs"]
mod supersession_retires_pending_predecessor;
pub(super) use supersession_retires_pending_predecessor::supersession_retires_pending_predecessor;

pub(super) fn duplicate_retry_does_not_schedule_again(
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
    let source = observed(&request, "anchor-b");
    let intent = PlanarSourceAdjustment {
        scope_key: "anchor-b".to_owned(),
        replacement_y: length(2),
    };
    let outcome = request
        .mutate(intent.clone())
        .expect_source(source.clone())
        .idempotency(&10_008)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the first source operation must perform")
    };
    let mut performed = performed
        .start_required_outputs(&world.application, &request, controls())
        .unwrap_or_else(|failure| panic!("required outputs start: {:?}", failure.denial()));
    let duplicate = request
        .mutate(intent)
        .expect_source(source)
        .idempotency(&10_008)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        duplicate,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
        )
    ));
    let settled = settle(&world.application, &mut performed, &request);
    assert_eq!(
        request
            .at(settled.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-b".to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(4)
    );
}

pub(super) fn two_forks_preserve_predecessor_output(
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
    let root = world.application.current_world();
    let left = fork(&world.application, root);
    let right = fork(&world.application, root);
    let left_request = world
        .application
        .request(&principal, &scope)
        .on_branch(left);
    let right_request = world
        .application
        .request(&principal, &scope)
        .on_branch(right);
    let root_request = world.application.request(&principal, &scope);

    let left_prepared = prepare(&left_request, &world.application, "anchor-c", 2, 10_009);
    let right_prepared = prepare(&right_request, &world.application, "anchor-c", 4, 10_010);
    let mut left_output = start(left_prepared, &world.application, &left_request);
    let mut right_output = start(right_prepared, &world.application, &right_request);
    let left_settled = settle(&world.application, &mut left_output, &left_request);
    let right_settled = settle(&world.application, &mut right_output, &right_request);
    assert_eq!(read_at(&left_request, &left_settled, "anchor-c"), length(4));
    assert_eq!(
        read_at(&right_request, &right_settled, "anchor-c"),
        length(6)
    );
    assert!(matches!(
        left_request
            .at(right_settled.observation())
            .require_current_program_output(&right_settled, NonZeroUsize::new(4_096).unwrap()),
        Err(WorthQueryProgramOutputCurrentnessDenial::ForeignBranch)
    ));
    assert_eq!(
        root_request
            .query(PlanarRead {
                body_key: "anchor-c".to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(10)
    );
    drop(left_settled);
    drop(right_settled);
    drop(left_output);
    drop(right_output);
    world
        .application
        .with_host_advancement(|host_phase| {
            world
                .application
                .on_branch(left)
                .close(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
    world
        .application
        .with_host_advancement(|host_phase| {
            world
                .application
                .on_branch(right)
                .close(&host_phase)
                .unwrap()
        })
        .expect("installed host call opens");
}

pub(super) fn branch_close_wakes_live_required_output(
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
    let branch = fork(&world.application, world.application.current_world());
    let request = world
        .application
        .request(&principal, &scope)
        .on_branch(branch);
    let mut output = start(
        prepare(&request, &world.application, "anchor-b", 2, 10_013),
        &world.application,
        &request,
    );
    let notifications = output.required_output_mut().notifications().unwrap();
    let before = notifications.generation();
    let pending_cleanup = match world.application.with_host_advancement(|host_phase| world.application.on_branch(branch).close(&host_phase)).expect("installed host call opens") {
        Ok(_) => None,
        Err(worth_query_host::facade::primary_graph::WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(
            failure,
        )) => Some(failure.into_cleanup()),
        Err(denial) => panic!("branch retirement reaches owner cleanup: {denial:?}"),
    };
    assert!(notifications.generation() > before);
    drop(output);
    if let Some(cleanup) = pending_cleanup {
        cleanup.retry().unwrap();
    }
}

type Request<'a> = worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
    'a,
    'a,
    'a,
    ConsumerSchema,
>;

type ProgramApplication =
    worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        ConsumerSchema,
        crate::ConsumerProgram,
    >;

type Prepared = worth_query_host::facade::application_entry::WorthQueryPerformedApplicationMutation<
    ConsumerSchema,
    PlanarSourceAdjustment,
    crate::ConsumerProgram,
    crate::ConsumerProgramRoot,
>;

type Started = worth_query_host::facade::application_entry::WorthQueryStartedRequiredOutputs<
    ConsumerSchema,
    PlanarSourceAdjustment,
    crate::ConsumerProgram,
    crate::ConsumerProgramRoot,
>;

pub(super) fn perform<'a>(
    request: &'a Request<'a>,
    application: &'a ProgramApplication,
    key: &str,
    y: u64,
    command: u64,
) -> Started {
    start(
        prepare(request, application, key, y, command),
        application,
        request,
    )
}

fn prepare<'a>(
    request: &'a Request<'a>,
    application: &'a ProgramApplication,
    key: &str,
    y: u64,
    command: u64,
) -> Prepared {
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: key.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(observed(request, key))
        .idempotency(&command)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the source operation reaches its installed program");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source operation must perform")
    };
    performed
}

fn start<'a>(
    performed: Prepared,
    application: &ProgramApplication,
    request: &'a Request<'a>,
) -> Started {
    performed
        .start_required_outputs(application, request, controls())
        .unwrap_or_else(|failure| panic!("required outputs start: {:?}", failure.denial()))
}

fn observed(request: &Request<'_>, key: &str) -> worth_query_host::facade::primary_graph::WorthQueryObservedSource<<worth_query_topology_entry::PlanarReadBinding<ConsumerSchema> as worth_query_decl::facade::application_query::ApplicationQueryBinding<ConsumerSchema>>::Query>{
    request
        .query(PlanarRead {
            body_key: key.to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone()
}

pub(super) fn controls() -> WorthQueryOutputDemandControls {
    WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    )
}

fn settle<'a>(
    application: &ProgramApplication,
    performed: &mut Started,
    request: &'a Request<'a>,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputSettlement<
    <worth_query_topology_entry::PlanarReadBinding<ConsumerSchema> as worth_query_decl::facade::application_query::ApplicationQueryBinding<ConsumerSchema>>::Query,
>{
    crate::application_invariant_acceptance::proof::settle(|| {
        crate::application_invariant_acceptance::proof::settled(
            performed
                .required_output_mut()
                .advance(application, request)
                .unwrap(),
        )
    })
}

fn read_at(
    request: &Request<'_>,
    settled: &worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputSettlement<
        <worth_query_topology_entry::PlanarReadBinding<ConsumerSchema> as worth_query_decl::facade::application_query::ApplicationQueryBinding<ConsumerSchema>>::Query,
    >,
    key: &str,
) -> worth_query_consumer_values::PositiveLength {
    request
        .at(settled.observation())
        .query(PlanarOutputRead {
            body_key: format!("final:{key}"),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .value
}
