use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
    WorthQueryApplicationRequestExt, WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::{
    PlanarOutputDemand, PlanarOutputRead, PlanarRead, PlanarSourceAdjustment,
};

use super::super::super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

pub(super) fn abandoned_and_superseded_preparations_are_bounded(
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

    let abandoned = prepare(&request, &world.application, "anchor-a", 2, 10_014);
    let abandoned_receipt = abandoned.receipt().clone();
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1
    );
    drop(abandoned);
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1,
        "caller disposal leaves exact prepared custody with the owner"
    );
    let mut recovered = request
        .recover_required_outputs::<crate::ConsumerProgram>(
            &world.application,
            &abandoned_receipt,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("owner custody re-enters the installed program");
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        0,
        "recovery consumes the exact prepared carrier"
    );
    let recovered_settlement = loop {
        match recovered.advance(&request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(
        request
            .at(recovered_settlement.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-a".to_owned(),
            })
            .execute()
            .expect("the recovered prepared carrier produces its final output")
            .rows()[0]
            .value,
        length(4)
    );

    drop(recovered_settlement);
    drop(recovered);
    drop(request);
    drop(world);
    let world = installation::install(foreign);
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the supersession proof authenticates its principal");
    let request = world.application.request(&principal, &scope);

    let older = prepare(&request, &world.application, "anchor-b", 2, 10_015);
    let successor = prepare(&request, &world.application, "anchor-b", 4, 10_016);
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1,
        "the successor retires older unconsumed custody"
    );
    let failure = older
        .start_required_outputs(&request, controls())
        .err()
        .expect("the retired predecessor cannot consume successor custody");
    let worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
        worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial),
    ) = failure.denial()
    else {
        panic!("superseded preparation must preserve the exact output-demand cause")
    };
    assert_eq!(
        denial.kind(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded
    );
    drop(failure);

    let mut successor = successor
        .start_required_outputs(&request, controls())
        .unwrap_or_else(|failure| panic!("successor starts: {:?}", failure.denial()));
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        0
    );
    let settled = settle(&mut successor, &request);
    assert_eq!(
        request
            .at(settled.observation())
            .query(PlanarOutputRead {
                body_key: "final:anchor-b".to_owned(),
            })
            .execute()
            .expect("the successor final output is retained")
            .rows()[0]
            .value,
        length(6)
    );
}

pub(super) fn close_before_required_output_start_is_typed(
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
    let branch = world
        .application
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the test branch is available");
    let request = world
        .application
        .request(&principal, &scope)
        .on_branch(branch);
    let prepared = prepare(&request, &world.application, "anchor-c", 2, 10_017);
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1
    );
    let pending_cleanup = match world.application.on_branch(branch).close() {
        Ok(_) => None,
        Err(worth_query_host::facade::primary_graph::WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(
            failure,
        )) => Some(failure.into_cleanup()),
        Err(denial) => panic!("branch retirement reaches owner cleanup: {denial:?}"),
    };
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        0
    );
    let failure = prepared
        .start_required_outputs(&request, controls())
        .err()
        .expect("retired occurrence denies its prepared source");
    let worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::SourceQuery(
        worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ProductSelection(denial),
    ) = failure.denial()
    else {
        panic!("retired occurrence must preserve the exact product-selection cause")
    };
    assert_eq!(
        *denial,
        worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial::RetiredBranch
    );
    drop(failure);
    drop(request);
    if let Some(cleanup) = pending_cleanup {
        cleanup
            .retry()
            .expect("released retained source permits exact owner cleanup");
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

fn prepare<'a>(
    request: &'a Request<'a>,
    application: &'a ProgramApplication,
    key: &str,
    y: u64,
    command: u64,
) -> Prepared<'a> {
    let source = request
        .query(PlanarRead {
            body_key: key.to_owned(),
        })
        .execute()
        .expect("the source is readable")
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: key.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(source)
        .idempotency(&command)
        .execute_performed(application)
        .expect("the source operation reaches its installed program");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source operation must perform")
    };
    performed
}

fn controls() -> WorthQueryOutputDemandControls {
    WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    )
}

fn settle<'a>(
    performed: &mut worth_query_host::facade::application_entry::WorthQueryStartedRequiredOutputs<
        'a,
        ConsumerSchema,
        PlanarSourceAdjustment,
        crate::ConsumerProgram,
    >,
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
