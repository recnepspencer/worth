//! Finite current delivery, consumed-fact staleness, and explicit no-projection ownership.
use super::*;
use crate::application_program::{
    no_projection::{NoProjectionAction, NoProjectionProgram},
    ConsumerFixedAction,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationDeliveryDenial,
    WorthQueryOperationDeliveryPolicy,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};

macro_rules! settle {
    ($request:expr, $world:expr, $program:ty, $action:ty, $input:expr, $key:expr) => {{
        let input = $input;
        let source = $request
            .query(PlanarRead {
                body_key: input.scope_key.clone(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        let mut operation = Some(
            $request
                .mutate(input)
                .expect_source(source)
                .idempotency_owned($key)
                .retain_operation::<$program, $action>(&$world.application)
                .unwrap_or_else(|(_, denial)| panic!("construction: {denial:?}")),
        );
        (0..32)
            .find_map(|_| match operation.take().unwrap().advance(&$request) {
                Advance::Progress {
                    operation: next, ..
                } => {
                    operation = Some(next);
                    None
                }
                Advance::Settled(settled) => Some(settled),
                Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                    panic!("native progression: {denial:?}")
                }
            })
            .expect("authored fixed graph settles")
    }};
}

#[test]
fn fixed_delivery_finite_retry_unrelated_progress_and_consumed_fact_history() {
    let world = installation::install(&crate::installed_schema());
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let original = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&original);
    let settled = settle!(
        request,
        world,
        crate::ConsumerProgram,
        ConsumerFixedAction,
        original.clone(),
        10_121
    );
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let one = NonZeroUsize::new(1).unwrap();
    let policy = WorthQueryOperationDeliveryPolicy::currentness(
        NonZeroUsize::new(
            world
                .application
                .runtime()
                .output_demand_resource_profile()
                .limits()
                .source_currentness_work(),
        )
        .unwrap(),
    );
    assert!(matches!(
        settled.read_current_with_policy(
            &request,
            WorthQueryOperationDeliveryPolicy::currentness(one)
        ),
        Err(WorthQueryOperationDeliveryDenial::Currentness(_))
    ));
    assert!(matches!(
        settled.read_current_with_policy(&request, policy.query(one, one)),
        Err(WorthQueryOperationDeliveryDenial::Query(worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Admission(denial)))
        if matches!(denial.kind(), worth_query_host::facade::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded)
    ));
    assert!(matches!(settled.read_current_with_policy(&request, policy.query(one, NonZeroUsize::new(129).unwrap())), Err(WorthQueryOperationDeliveryDenial::Query(worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Limit(_)))));
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let request = world.application.request(&fresh_principal, &fresh_scope);
    let read = settled.read_current_with_policy(&request, policy).unwrap();
    assert_eq!(read.rows()[0].value, length(3));
    drop(read);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head,
        "delivery refusals and retry publish no source or output"
    );
    assert_eq!(
        settled.settlement().root_producer_contacts_in_this_demand(),
        1
    );
    let unrelated = settle!(
        request,
        world,
        crate::ConsumerProgram,
        ConsumerFixedAction,
        PlanarSourceAdjustment {
            scope_key: "remote-a".to_owned(),
            replacement_y: length(42)
        },
        10_122
    );
    unrelated
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("unrelated close: {denial:?}"));
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(3)
    );
    assert_eq!(planar_source_adjustment_contacts(&original), (1, 1));
    let changed = settle!(
        request,
        world,
        crate::ConsumerProgram,
        ConsumerFixedAction,
        PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(4)
        },
        10_123
    );
    changed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("changed close: {denial:?}"));
    assert!(matches!(
        settled.read_current(&request),
        Err(WorthQueryOperationDeliveryDenial::Stale(_))
    ));
    assert_eq!(
        settled.read_historical(&request).unwrap().rows()[0].value,
        length(3)
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("stale close: {denial:?}"));
    assert_eq!(planar_source_adjustment_contacts(&original), (1, 1));
}

#[test]
fn fixed_delivery_no_projection_settles_real_required_outputs() {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, NoProjectionProgram>::begin()
        .validated_program()
        .unwrap();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<NoProjectionProgram>(
            "model-owner",
            "model-owner",
        ),
    ];
    let world = installation::install_selected_program(
        &crate::installed_schema(),
        8192,
        4096,
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &grants,
        program,
    )
    .unwrap();
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let settled = settle!(
        request,
        world,
        NoProjectionProgram,
        NoProjectionAction,
        input.clone(),
        10_124
    );
    assert_eq!(settled.settlement().output_count(), 4);
    assert_eq!(
        settled.settlement().root_producer_contacts_in_this_demand(),
        1
    );
    let before =
        worth_query_host::facade::primary_graph::query_read_kernel_entries_on_this_thread_for_test(
        );
    let observation = settled
        .deliver_current(
            &request,
            WorthQueryOperationDeliveryPolicy::currentness(
                NonZeroUsize::new(
                    world
                        .application
                        .runtime()
                        .output_demand_resource_profile()
                        .limits()
                        .source_currentness_work(),
                )
                .unwrap(),
            ),
        )
        .unwrap();
    assert_eq!(
        worth_query_host::facade::primary_graph::query_read_kernel_entries_on_this_thread_for_test(
        ),
        before,
        "no-projection delivery enters no query kernel"
    );
    assert_eq!(
        request
            .at(&observation)
            .query(worth_query_topology_entry::PlanarOutputRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(3)
    );
    drop(observation);
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("no-projection close: {denial:?}"));
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
}
