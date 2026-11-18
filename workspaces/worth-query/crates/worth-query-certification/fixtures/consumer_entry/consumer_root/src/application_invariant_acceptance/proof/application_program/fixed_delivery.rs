//! Finite current delivery, consumed-fact staleness, and explicit no-projection ownership.
use super::*;
use crate::application_program::{
    no_projection::{NoProjectionAction, NoProjectionProgram},
    ConsumerFixedAction, ConsumerProgram,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationDeliveryDenial,
    WorthQueryOperationDeliveryPolicy,
};
use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster as ProgramRoster;
use worth_query_host::facade::primary_graph::{self, WorthQueryExecutionObservation};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts, FinalAnchorOutput,
    FinalPlanarOutputs, PlanarAlternateFinalToSummaryConnection, PlanarAnchorOutput,
    PlanarFinalToSummaryConnection, PlanarOutputRead, PlanarOutputToAlternateFinalConnection,
    PlanarOutputToFinalConnection, PlanarOutputs,
};

macro_rules! settle {
    ($request:expr, $world:expr, $program:ty, $action:ty, $input:expr, $key:expr) => {{
        let before = $request.retain_current_read().unwrap();
        let before = before.selected_commit();
        let input = $input;
        let source = $request
            .query(PlanarRead {
                body_key: input.scope_key.clone(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        let root = source.root_entity_for_test();
        let mut operation = Some(
            $request
                .mutate(input)
                .expect_source(source)
                .idempotency_owned($key)
                .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
                .retain_operation::<$program, $action>(&$world.application)
                .unwrap_or_else(|(_, denial)| panic!("construction: {denial:?}")),
        );
        let settled = (0..32)
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
            .expect("authored fixed graph settles");
        let after = $request.retain_current_read().unwrap();
        let after = after.selected_commit();
        assert_eq!(after.owner_identity(), before.owner_identity());
        assert_eq!(after.ordinal(), before.ordinal() + 1);
        let outputs = settled.settlement();
        assert_eq!(
            outputs.output_count(),
            4,
            "every required graph position settles"
        );
        let fulfillment = outputs.performed_source().unwrap().root_fulfillment();
        assert_eq!(fulfillment.source_commit(), after);
        assert_eq!(fulfillment.output_commit(), after);
        assert_eq!(outputs.observation().selected_commit(), after);
        let root_settlement = outputs.root_settlement();
        let facts = root_settlement.outputs_of::<PlanarOutputs>().unwrap();
        type Anchor = PlanarAnchorOutput<ConsumerSchema>;
        let anchor = facts.entity::<Anchor>().unwrap();
        assert_eq!(anchor.entity_id(), root);
        settled
    }};
}

mod retry_history;

#[test]
fn fixed_delivery_no_projection_settles_real_required_outputs() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let program = ApplicationProgramAuthoring::<ConsumerSchema, NoProjectionProgram>::begin()
        .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the completed schema is valid"))
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
        ProgramRoster::new(),
        &grants,
        program,
    )
    .unwrap();
    let app = &world.application;
    let adapter = authentication::admit(app.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = app.request(&principal, &scope);
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
    let outputs = settled.settlement();
    assert_eq!(outputs.output_count(), 4);
    assert_eq!(outputs.root_producer_contacts_in_this_demand(), 0);
    let head = request.retain_current_read().unwrap();
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let before = primary_graph::query_read_kernel_entries_on_this_thread_for_test();
    let observation = settled.deliver_current(&request).unwrap();
    assert_eq!(
        primary_graph::query_read_kernel_entries_on_this_thread_for_test(),
        before,
        "no-projection delivery enters no query kernel"
    );
    let query = request.at(&observation).query(PlanarOutputRead {
        body_key: "anchor-a".into(),
    });
    let value = query.execute().unwrap();
    assert_eq!(value.rows()[0].value, length(3));
    drop(value);
    drop(observation);
    assert_eq!(app.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        head.selected_commit()
    );
    drop(head);
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
        .with_schema(&<ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchema>::declaration().expect("the completed schema is valid"))
        .validated_program()
        .unwrap();
    let mut world = installation::install_selected_program(
        &crate::installed_schema(),
        ProgramRoster::new(),
        &source::deployment_grants(),
        program,
    )
    .unwrap();
    let app = &world.application;
    let source_binding = app
        .installed_schema()
        .installed_mutation_binding::<PlanarEditBinding<ConsumerSchema>>()
        .unwrap();
    let adapter = authentication::admit(app.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = app.request(&principal, &scope);
    // Each installed binding registers once per original co-publication: slot zero.
    let describe = |receipt: &WorthQueryApplicationCommitReceipt| {
        let admitted = receipt.principal_scope();
        let scope = admitted.scope();
        let publication = receipt.committed_product_publication();
        (
            admitted.runtime_authority(),
            (scope.partition_id(), scope.local_slot(), scope.generation()),
            source_binding.identity().to_owned(),
            publication.product_incarnation(),
            publication.product_generation().get(),
            0,
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
        let target_root = request
            .query(PlanarRead {
                body_key: target.into(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .root_entity_for_test();
        let before = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let contacts = app.producer_contacts_on_this_thread_for_test();
        let operation = source::prepare(app, &request, source::input(target, length(y)), key);
        assert_eq!(
            app.producer_contacts_on_this_thread_for_test(),
            contacts + 1
        );
        let receipt = operation.prepared_source_receipt().unwrap().clone();
        let publication = receipt.committed_product_publication();
        let commit = publication.composite_commit();
        assert_eq!(commit.owner_identity(), before.owner_identity());
        assert_eq!(commit.ordinal(), before.ordinal() + 1);
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            commit
        );
        assert_eq!(
            receipt.principal_scope().operation_authority_identity(),
            source_binding.operation().authority_identity()
        );
        source::assert_y(&request, target, length(y));
        assert_eq!(
            request
                .query(PlanarOutputRead {
                    body_key: target.into()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            length(y + 1)
        );
        windows.push((
            format!("{target}:source preparation"),
            primary_graph::native_marking_on_this_thread_for_test(),
        ));
        sources.push(describe(&receipt));
        let settled = source::finish(operation, &request);
        let outputs = settled.settlement();
        assert_eq!(outputs.output_count(), 0, "root-only graph");
        assert_eq!(outputs.root_producer_contacts_in_this_demand(), 0);
        let performed = outputs.performed_source().unwrap();
        assert!(performed
            .receipt()
            .retained_receipt()
            .unwrap()
            .is_same_authoritative_commit(&receipt));
        let fulfillment = performed.root_fulfillment();
        assert!(fulfillment.matches_source(&receipt));
        assert_eq!(fulfillment.source_commit(), commit);
        assert_eq!(fulfillment.output_commit(), commit);
        let root = outputs.root_settlement();
        let facts = root.outputs_of::<PlanarOutputs>().unwrap();
        assert_eq!(
            facts
                .entity::<PlanarAnchorOutput<ConsumerSchema>>()
                .unwrap()
                .entity_id(),
            target_root
        );
        assert_eq!(root.observed_source().root_entity_for_test(), target_root);
        assert_eq!(root.observation().selected_commit(), commit);
        assert_eq!(outputs.observation().selected_commit(), commit);
        // Descriptive coordinates only: the installed producer, actual typed root,
        // and original co-publication define membership independently of marking.
        producers.push((
            receipt.principal_scope().runtime_authority(),
            (
                target_root.partition_value(),
                target_root.local_slot_value(),
                target_root.generation_value(),
            ),
            PlanarMutationBinding::<ConsumerSchema>::IDENTITY.to_owned(),
            publication.product_incarnation(),
            publication.product_generation().get(),
            0,
        ));
        settled
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("same original owner close: {cause:?}"));
        assert_eq!(
            app.producer_contacts_on_this_thread_for_test(),
            contacts + 1
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            commit
        );
        windows.push((
            format!("{target}:output completion and close"),
            primary_graph::native_marking_on_this_thread_for_test(),
        ));
        source::assert_y(&request, target, length(y));
    }
    assert_eq!(sources.len(), 2);
    assert_eq!(producers.len(), 2);
    assert_eq!(sources[0].0, sources[1].0);
    assert_eq!(sources[0].1, sources[1].1);
    assert_eq!(sources[0].2, sources[1].2);
    assert_eq!(sources[0].3, sources[1].3);
    assert_ne!(sources[0].4, sources[1].4, "distinct original publications");
    assert_ne!(sources[0], producers[0], "Source is not producer evidence");
    assert_ne!(sources[1], producers[1]);
    assert_ne!(sources[0].2, producers[0].2);
    assert_ne!(producers[0], producers[1]);
    let edit = source::edit_target(app, &request);
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
                let scope = marked.root;
                actual.push((
                    marked.runtime_authority,
                    (scope.partition_id(), scope.local_slot(), scope.generation()),
                    marked.output_binding.to_string(),
                    marked.product_incarnation,
                    marked.generation,
                    marked.slot,
                ));
            }
        }
        actual.sort();
        assert!(
            actual.windows(2).all(|pair| pair[0] != pair[1]),
            "ambiguous projected marking membership"
        );
        assert!(actual
            .iter()
            .all(|item| sources.contains(item) || producers.contains(item)));
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
    // The Y edit newly dirties producer0, which reads Y and writes Length.
    // The matching sources read and replace Y, so start dirty at publication.
    assert_eq!(
        actual,
        vec![producers[0].clone()],
        "complete independent semantic multiset, including multiplicity"
    );
    assert!(
        !actual.contains(&sources[1]) && !actual.contains(&producers[1]),
        "same-scope sibling excluded"
    );
    source::assert_y(&request, "sibling-a", length(23));
    source::assert_y(&request, "remote-a", length(42));
    world.application.close_conditional_runtime().unwrap();
}
