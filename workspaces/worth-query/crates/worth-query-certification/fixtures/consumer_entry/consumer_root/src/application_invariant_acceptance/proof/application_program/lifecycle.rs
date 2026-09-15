use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{
    PlanarOutputRead, PlanarRead, PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

pub(super) fn supersession_retires_pending_predecessor(
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

    let mut predecessor = perform(&request, &world.application, "anchor-a", 2, 10_006);
    assert!(matches!(
        predecessor.required_output_mut().advance(&request).unwrap(),
        WorthQueryApplicationProgramOutputProgress::Pending
    ));
    world
        .application
        .delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        predecessor.required_output_mut().advance(&request).unwrap(),
        WorthQueryApplicationProgramOutputProgress::Pending
    ));

    let mut successor = perform(&request, &world.application, "anchor-a", 5, 10_007);
    assert!(matches!(
        predecessor.required_output_mut().advance(&request),
        Err(worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
            WorthQueryApplicationOutputDemandDenial::Superseded,
        ))
    ));
    let settled = settle(&mut successor, &request);
    let exact = request
        .at(settled.observation())
        .query(PlanarOutputRead {
            body_key: "final:anchor-a".to_owned(),
        })
        .execute()
        .expect("the successor output remains exactly readable");
    assert_eq!(exact.rows()[0].value, length(7));
}

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
        .execute_performed(&world.application)
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the first source operation must perform")
    };
    let mut performed = performed
        .start_required_outputs(&request, controls())
        .unwrap_or_else(|failure| panic!("required outputs start: {:?}", failure.denial()));
    let duplicate = request
        .mutate(intent)
        .expect_source(source)
        .idempotency(&10_008)
        .execute_performed(&world.application)
        .unwrap();
    assert!(matches!(
        duplicate,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
        )
    ));
    let settled = settle(&mut performed, &request);
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
    let mut left_output = start(left_prepared, &left_request);
    let mut right_output = start(right_prepared, &right_request);
    let left_settled = settle(&mut left_output, &left_request);
    let right_settled = settle(&mut right_output, &right_request);
    assert_eq!(read_at(&left_request, &left_settled, "anchor-c"), length(4));
    assert_eq!(
        read_at(&right_request, &right_settled, "anchor-c"),
        length(6)
    );
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
    drop(left_request);
    drop(right_request);
    world.application.on_branch(left).close().unwrap();
    world.application.on_branch(right).close().unwrap();
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
        &request,
    );
    let notifications = output.required_output_mut().notifications().unwrap();
    let before = notifications.generation();
    let pending_cleanup = match world.application.on_branch(branch).close() {
        Ok(_) => None,
        Err(worth_query_host::facade::primary_graph::WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(
            failure,
        )) => Some(failure.into_cleanup()),
        Err(denial) => panic!("branch retirement reaches owner cleanup: {denial:?}"),
    };
    assert!(notifications.generation() > before);
    drop(output);
    drop(request);
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

type Prepared<'a> =
    worth_query_host::facade::application_entry::WorthQueryPerformedApplicationMutation<
        'a,
        ConsumerSchema,
        PlanarSourceAdjustment,
        crate::ConsumerProgram,
    >;

type Started<'a> = worth_query_host::facade::application_entry::WorthQueryStartedRequiredOutputs<
    'a,
    ConsumerSchema,
    PlanarSourceAdjustment,
    crate::ConsumerProgram,
>;

pub(super) fn perform<'a>(
    request: &'a Request<'a>,
    application: &'a ProgramApplication,
    key: &str,
    y: u64,
    command: u64,
) -> Started<'a> {
    start(prepare(request, application, key, y, command), request)
}

fn prepare<'a>(
    request: &'a Request<'a>,
    application: &'a ProgramApplication,
    key: &str,
    y: u64,
    command: u64,
) -> Prepared<'a> {
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: key.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(observed(request, key))
        .idempotency(&command)
        .execute_performed(application)
        .expect("the source operation reaches its installed program");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source operation must perform")
    };
    performed
}

fn start<'a>(performed: Prepared<'a>, request: &'a Request<'a>) -> Started<'a> {
    performed
        .start_required_outputs(request, controls())
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
    performed: &mut Started<'a>,
    request: &'a Request<'a>,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationProgramOutputSettlement<
    <worth_query_topology_entry::PlanarReadBinding<ConsumerSchema> as worth_query_decl::facade::application_query::ApplicationQueryBinding<ConsumerSchema>>::Query,
>{
    loop {
        match performed.required_output_mut().advance(request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => return settled,
        }
    }
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

fn fork(
    application: &worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        ConsumerSchema,
    >,
    source: worth_query_host::facade::product::WorthQueryProductBranch,
) -> worth_query_host::facade::product::WorthQueryProductBranch {
    application
        .branches()
        .fork(source)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .unwrap()
}
