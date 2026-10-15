//! Isolated domain progress preserves complete currentness; consumed facts retain original history.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryOperationDeliveryDenial as DeliveryDenial;
#[test]
fn discovered_ordinary_unrelated_progress_is_current_and_consumed_fact_keeps_history() {
    let original = ApplicationProgramAuthoring::<ConsumerSchema, Program<QueryDelivery>>::begin()
        .validated_program()
        .unwrap();
    let world = installation::install_selected_program_with_isolated_cycle(
        &crate::installed_schema(),
        8192,
        4096,
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new(
        ),
        &[],
        true,
        original,
    )
    .unwrap();
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let original_input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&original_input);
    let mut operation = Some(
        request
            .mutate(original_input.clone())
            .expect_source(source)
            .idempotency_owned(110_034)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(&world.application)
            .unwrap_or_else(|(_, d)| panic!("construction: {d:?}")),
    );
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&request) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            _ => panic!("ordinary fanout"),
        })
        .unwrap();
    let original_commit = settled.settlement().observation().selected_commit().clone();
    let original_basis = settled
        .read_historical(&request)
        .unwrap()
        .receipt()
        .inspect()
        .basis()
        .clone();
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    for (key, replacement, expected_before, id) in [
        ("isolated-a", 62, 61, 110_035),
        ("remote-a", 42, 41, 110_036),
    ] {
        let input = PlanarSourceAdjustment {
            scope_key: key.to_owned(),
            replacement_y: length(replacement),
        };
        reset_planar_source_adjustment_contacts(&input);
        let read = request
            .query(PlanarRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        assert_eq!(read.rows()[0].y, length(expected_before));
        let source = read.observed_sources()[0].clone();
        drop(read);
        let mut next = Some(
            request
                .mutate(input.clone())
                .expect_source(source)
                .idempotency_owned(id)
                .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
                .retain_operation::<Program<QueryDelivery>, Action<QueryDelivery>>(
                    &world.application,
                )
                .unwrap_or_else(|(_, d)| panic!("construction: {d:?}")),
        );
        let changed = (0..32)
            .find_map(|_| match next.take().unwrap().advance(&request) {
                Advance::Progress { operation, .. } => {
                    next = Some(operation);
                    None
                }
                Advance::Settled(settled) => Some(settled),
                Advance::Refused { denial, .. } => panic!("{key} native refusal: {denial:?}"),
                Advance::Terminal { operation, denial } => panic!(
                    "{key} terminal: {denial:?}, native: {:?}",
                    operation.native_outcome()
                ),
            })
            .unwrap();
        changed
            .close(&request)
            .unwrap_or_else(|(_, d)| panic!("close: {d:?}"));
        let read = request
            .query(PlanarRead {
                body_key: key.to_owned(),
            })
            .execute()
            .unwrap();
        assert_eq!(read.rows()[0].y, length(replacement));
        assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
        assert_ne!(
            request.retain_current_read().unwrap().selected_commit(),
            &before
        );
        if key == "isolated-a" {
            assert_eq!(
                settled.read_current(&request).unwrap().rows()[0].value,
                length(42)
            );
        }
    }
    assert_eq!(planar_source_adjustment_contacts(&original_input), (1, 1));
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    assert!(
        matches!(settled.read_current(&request),Err(DeliveryDenial::Stale(denial)) if denial.kind()==worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded)
    );
    let history = settled.read_historical(&request).unwrap();
    assert_eq!(history.rows()[0].value, length(42));
    let actual = history.receipt().inspect().basis().clone();
    assert_eq!(actual.runtime_instance(), original_basis.runtime_instance());
    assert_eq!(actual.branch(), original_basis.branch());
    assert_eq!(actual.version(), original_basis.version());
    assert_eq!(actual.posture(), original_basis.posture());
    assert_eq!(
        settled.settlement().observation().selected_commit(),
        &original_commit
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("old owner close: {d:?}"));
}
