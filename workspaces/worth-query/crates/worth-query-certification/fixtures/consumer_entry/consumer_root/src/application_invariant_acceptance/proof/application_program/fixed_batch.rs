//! Installed two-selector fixed batch under one native allowance and observation.
use super::*;
use crate::application_program::fixed_batch::{
    BatchAction, BatchDelivery, BatchProgram, CounterfeitBatchAction, CounterfeitBatchDelivery,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationQueryBatchDenial, WorthQueryOperationAdvance as Advance,
    WorthQueryOperationDeliveryDenial, WorthQueryOperationDeliveryPolicy,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationQueryBatchLimits, WorthQueryApplicationQueryBatchResourceDenial,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts, PlanarOutputRead,
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
fn fixed_batch_aggregate_refusal_and_fresh_retry_preserve_settled_owner() {
    let program = ApplicationProgramAuthoring::<ConsumerSchema, BatchProgram>::begin()
        .validated_program()
        .unwrap();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<BatchProgram>(
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
    use worth_query_decl::facade::application_program::ApplicationActionDeliveryDeclaration;
    use worth_query_topology_entry::PlanarSourceAdjustmentBinding;
    assert_eq!(
        ApplicationActionDeliveryDeclaration::fixed::<
            ConsumerSchema,
            PlanarSourceAdjustmentBinding<ConsumerSchema>,
            BatchDelivery,
        >()
        .canonical_meaning(),
        ApplicationActionDeliveryDeclaration::fixed::<
            ConsumerSchema,
            PlanarSourceAdjustmentBinding<ConsumerSchema>,
            CounterfeitBatchDelivery,
        >()
        .canonical_meaning()
    );
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let description = request
        .mutate(input.clone())
        .expect_source(source)
        .idempotency_owned(10_142)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"));
    let (description, denial) = match description
        .retain_operation::<BatchProgram, CounterfeitBatchAction>(&world.application)
    {
        Err(refusal) => refusal,
        Ok(_) => panic!("counterfeit batch selector admitted"),
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryOperationConstructionDenial::Action(
            worth_query_host::facade::domain::WorthQueryActionResolutionDenial::DeliveryMismatch
        )
    ));
    assert_eq!(planar_source_adjustment_contacts(&input), (0, 0));
    let valid = description
        .retain_operation::<BatchProgram, BatchAction>(&world.application)
        .unwrap_or_else(|(_, denial)| panic!("valid twin: {denial:?}"));
    drop(valid);
    let settled = settle!(
        request,
        world,
        BatchProgram,
        BatchAction,
        input.clone(),
        10_141
    );

    assert_eq!(settled.settlement().output_count(), 4);
    let basis = settled.settlement().observation();
    let retained = request.at(basis);
    let mut works = Vec::new();
    let mut retained_basis = None;
    for key in ["anchor-a", "final:anchor-a"] {
        let scalar = retained
            .query(PlanarOutputRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        retained_basis.get_or_insert_with(|| scalar.receipt().inspect().basis().clone());
        works.push(scalar.receipt().inspect().read_work().total_work_units());
    }
    // Fund currentness independently so these refusals reach aggregate batch
    // admission rather than exhausting an unrelated earlier proof allowance.
    let policy = WorthQueryOperationDeliveryPolicy::currentness(
        NonZeroUsize::new(
            world
                .application
                .runtime()
                .output_demand_resource_profile()
                .limits()
                .source_currentness_work(),
        )
        .expect("installed finite positive currentness allowance"),
    );
    let cap = works.iter().copied().max().unwrap();
    assert!(cap < works.iter().sum::<usize>());
    let limits = WorthQueryApplicationQueryBatchLimits::new(
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::new(cap).unwrap(),
        NonZeroUsize::MAX,
        NonZeroUsize::MAX,
    );
    let observer = world.application.runtime().result_buffer_observer();
    let before = observer.observe();
    let custody_cap = WorthQueryApplicationQueryBatchLimits::new(
        NonZeroUsize::new(2).unwrap(),
        NonZeroUsize::MAX,
        NonZeroUsize::new(1).unwrap(),
        NonZeroUsize::MAX,
    );
    let memory_refusal =
        match settled.read_batch_current_with_policy::<2>(&request, policy.batch(custody_cap)) {
            Err(denial) => denial,
            Ok(_) => panic!("one byte cannot retain both batch result slots"),
        };
    assert!(
        matches!(
            memory_refusal,
            WorthQueryOperationDeliveryDenial::Batch(
                WorthQueryApplicationQueryBatchDenial::Resource {
                    item: None,
                    denial: WorthQueryApplicationQueryBatchResourceDenial::MemoryLimit {
                        maximum: 1,
                        ..
                    }
                }
            )
        ),
        "actual aggregate custody refusal: {memory_refusal:?}",
    );
    let refusal = match settled.read_batch_current_with_policy::<2>(&request, policy.batch(limits))
    {
        Err(denial) => denial,
        Ok(_) => panic!("aggregate allowance was renewed for the second item"),
    };
    assert!(
        matches!(
            refusal,
            WorthQueryOperationDeliveryDenial::Batch(
                WorthQueryApplicationQueryBatchDenial::Resource {
                    item: Some(1),
                    denial: WorthQueryApplicationQueryBatchResourceDenial::WorkLimit { .. }
                }
            )
        ),
        "actual native refusal: {refusal:?}"
    );
    assert_eq!(observer.observe().retained_bytes(), before.retained_bytes());
    assert_eq!(observer.observe().active_buffers(), before.active_buffers());
    let fresh_scope = authentication::request_scope();
    let fresh = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let request = world.application.request(&fresh, &fresh_scope);
    let batch = settled.read_batch_current::<2>(&request).unwrap();
    assert_eq!(batch.items().len(), 2);
    assert_eq!(batch.items()[0].rows()[0].value, length(3));
    assert_eq!(batch.items()[1].rows()[0].value, length(4));
    assert_eq!(
        batch.items()[0].receipt().inspect().basis(),
        batch.items()[1].receipt().inspect().basis()
    );
    let original_basis = batch.items()[0].receipt().inspect().basis().clone();
    drop(batch);
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let unrelated = settle!(
        request,
        world,
        BatchProgram,
        BatchAction,
        PlanarSourceAdjustment {
            scope_key: "remote-a".to_owned(),
            replacement_y: length(42)
        },
        10_143
    );
    unrelated
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("unrelated close: {denial:?}"));
    let current = settled.read_batch_current::<2>(&request).unwrap();
    let current_basis = current.items()[0].receipt().inspect().basis().clone();
    assert_ne!(current_basis, original_basis, "currentness and query cannot reuse the historical observation after unrelated native progress");
    assert_eq!(
        current.items()[1].receipt().inspect().basis(),
        &current_basis
    );
    assert_eq!(current.items()[0].rows()[0].value, length(3));
    assert_eq!(current.items()[1].rows()[0].value, length(4));
    drop(current);
    let historical = settled.read_batch_historical::<2>(&request).unwrap();
    for item in historical.items() {
        let actual = item.receipt().inspect();
        let expected = retained_basis.as_ref().unwrap();
        assert_eq!(
            actual.basis().runtime_instance(),
            expected.runtime_instance()
        );
        assert_eq!(actual.basis().branch(), expected.branch());
        assert_eq!(actual.basis().version(), expected.version());
        assert_eq!(actual.basis().posture(), expected.posture());
    }
    assert_eq!(
        historical.items()[0].receipt().inspect().basis(),
        historical.items()[1].receipt().inspect().basis()
    );
    assert_ne!(
        historical.items()[0].receipt().inspect().basis().version(),
        current_basis.version()
    );
    drop(historical);
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("close: {denial:?}"));
}

#[test]
fn fixed_batch_empty_declaration_refuses_before_execution() {
    let denial = match ApplicationProgramAuthoring::<
        ConsumerSchema,
        crate::application_program::fixed_batch::EmptyBatchProgram,
    >::begin()
    .validated_program()
    {
        Err(denial) => denial,
        Ok(_) => panic!("empty fixed batch accepted"),
    };
    assert_eq!(denial.kind(), worth_query_decl::facade::application_program::ApplicationProgramValidationDenialKind::InvalidActionDelivery);
}

#[test]
fn fixed_batch_cardinality_is_installed_program_meaning() {
    let two = ApplicationProgramAuthoring::<ConsumerSchema, BatchProgram>::begin()
        .validated_program()
        .unwrap();
    let three = ApplicationProgramAuthoring::<
        ConsumerSchema,
        crate::application_program::fixed_batch::TripleBatchProgram,
    >::begin()
    .validated_program()
    .unwrap();
    assert_ne!(
        two.revision(),
        three.revision(),
        "same action/query identities with a changed bound must change program meaning"
    );
}
