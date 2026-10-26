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
                .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
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
fn fixed_delivery_result_contract_retry_unrelated_progress_and_consumed_fact_history() {
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
    let _one = NonZeroUsize::new(1).unwrap();
    let policy = WorthQueryOperationDeliveryPolicy::default();

    assert!(matches!(settled.read_current_with_policy(&request, policy.query(NonZeroUsize::new(2).unwrap())), Err(WorthQueryOperationDeliveryDenial::Query(worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Limit(_)))));
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
    let observation = settled.deliver_current(&request).unwrap();
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

#[test]
fn fixed_prepared_source_receipts_distinguish_shared_scope_publications() {
    use crate::application_program::fixed_source_receipts as source;
    use worth_query_decl::facade::application_operation::ApplicationMutationBinding;
    use worth_query_host::facade::primary_graph::{self, WorthQueryApplicationCommitReceipt};
    use worth_query_topology_entry::{PlanarEditBinding, PlanarMutationBinding};
    let _observation = primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    source::assert_single_adjustment_shape();
    let program = ApplicationProgramAuthoring::<ConsumerSchema, source::Program>::begin()
        .validated_program()
        .unwrap();
    let mut world = installation::install_selected_program(
        &crate::installed_schema(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &source::deployment_grants(),
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
    let describe = |receipt: &WorthQueryApplicationCommitReceipt, binding: &str| {
        let admitted = receipt.principal_scope();
        let publication = receipt.committed_product_publication();
        (
            admitted.runtime_authority(),
            admitted.scope(),
            binding.to_owned(),
            publication.product_incarnation(),
            publication.product_generation().get(),
        )
    };
    source::assert_y(&request, "sibling-a", length(21));
    source::assert_y(&request, "remote-a", length(41));
    let mut windows = vec![(
        "installation".to_owned(),
        primary_graph::native_marking_on_this_thread_for_test(),
    )];
    let mut sources = Vec::new();
    let mut producers = Vec::new();
    for (target, y, key) in [("sibling-a", 22, 11_201), ("remote-a", 42, 11_202)] {
        let operation = source::prepare(
            &world.application,
            &request,
            source::input(target, length(y)),
            key,
        );
        windows.push((
            format!("{target}:source preparation"),
            primary_graph::native_marking_on_this_thread_for_test(),
        ));
        sources.push(describe(
            operation
                .prepared_source_receipt()
                .expect("borrow original source before output start"),
            PlanarEditBinding::<ConsumerSchema>::IDENTITY,
        ));
        let settled = source::finish(operation, &request);
        producers.push(describe(
            settled
                .settlement()
                .root_receipt()
                .expect("actual root producer receipt"),
            PlanarMutationBinding::<ConsumerSchema>::IDENTITY,
        ));
        settled
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("same original owner close: {cause:?}"));
        windows.push((
            format!("{target}:output completion and close"),
            primary_graph::native_marking_on_this_thread_for_test(),
        ));
        source::assert_y(&request, target, length(y));
    }
    source::assert_receipt_descriptions(&sources, &producers);
    let edit = source::edit_target(&world.application, &request);
    let measured = windows.len();
    windows.push((
        "follow-up:source preparation".to_owned(),
        primary_graph::native_marking_on_this_thread_for_test(),
    ));
    source::finish(edit, &request)
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("follow-up owner close: {cause:?}"));
    windows.push((
        "follow-up:output completion and close".to_owned(),
        primary_graph::native_marking_on_this_thread_for_test(),
    ));
    let mut measured_membership = None;
    for (window, (label, reports)) in windows.iter().enumerate() {
        let mut actual = Vec::new();
        for (index, report) in reports.iter().enumerate() {
            assert!(reports[..index]
                .iter()
                .all(|prior| prior.commit != report.commit));
            assert!(matches!(
                report.precision,
                primary_graph::WorthQueryNativeMarkingPrecision::Exact(_)
            ));
            for marked in report
                .affected
                .as_ref()
                .expect("exact semantic affected membership")
            {
                actual.push((
                    marked.runtime_authority,
                    marked.root,
                    marked.output_binding.to_string(),
                    marked.product_incarnation,
                    marked.generation,
                ));
            }
        }
        source::assert_known_window(&mut actual, &sources, &producers);
        println!(
            "{label}: {} reports; semantic membership {actual:?}",
            reports.len()
        );
        if window == measured {
            assert!(!reports.is_empty());
            measured_membership = Some(actual);
        }
    }
    let actual = measured_membership.expect("one separately retained source-edit window");
    source::assert_source_edit_membership(&actual, &sources, &producers);
    source::assert_y(&request, "sibling-a", length(23));
    source::assert_y(&request, "remote-a", length(42));
    world.application.close_conditional_runtime().unwrap();
}
