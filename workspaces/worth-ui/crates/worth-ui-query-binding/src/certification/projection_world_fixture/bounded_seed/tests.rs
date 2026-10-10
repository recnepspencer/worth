use std::collections::BTreeMap;

use super::*;
use crate::{
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionChange, UiCollectionProjectionFactReceipt,
    UiCollectionProjectionOpenOutcome, UiCollectionProjectionRefreshOutcome,
    UiCollectionProjectionRegistration, UiCollectionProjectionValue, UiLiveCollectionProjection,
    UiLiveCollectionProjectionCloseOutcome, UiPresentProjection, UiProjectionAvailability,
    UiProjectionFieldRequirement, WorthUiQueryWorkspaceExt,
};

#[test]
fn receipt_entities_correlate_to_authored_order_across_batches() {
    let row_count = 2 * MAX_ATOMIC_SEED_ROWS + 1;
    let (mut workspace, entities) = collection_projection_workspace(
        authored_rows(row_count),
        WorthUiCollectionProjectionSeedPosture::Complete,
    );
    assert_eq!(entities.len(), row_count);

    let (live, fact) = open_projection(&mut workspace, row_count as u32);
    let value = present(&fact);
    let query_entities_by_status = value
        .rows()
        .iter()
        .map(|row| {
            (
                row.selected_values()[0].as_str().to_owned(),
                row.row().query_identity().operational_key(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (index, entity) in entities.iter().enumerate() {
        assert_eq!(
            query_entities_by_status.get(&format!("Value {index:05}")),
            Some(&entity.evidence_identity().operational_key())
        );
    }
    let UiLiveCollectionProjectionCloseOutcome::Closed(closed) = live.close(&mut workspace) else {
        panic!("QP04 live collection owner must close");
    };
    assert!(closed.owner_terminal());
}

#[test]
fn ordinary_window_opens_512_of_2049_rows_and_maintains_tail_identity() {
    const VISIBLE_ROWS: usize = 512;

    let row_count = 2 * MAX_ATOMIC_SEED_ROWS + 1;
    let (mut workspace, entities) = collection_projection_workspace(
        authored_rows(row_count),
        WorthUiCollectionProjectionSeedPosture::Complete,
    );
    assert_eq!(entities.len(), row_count);

    let (mut live, initial) = open_projection(&mut workspace, VISIBLE_ROWS as u32);
    let initial_value = present(&initial);
    let initial_indices = (0..VISIBLE_ROWS).collect::<Vec<_>>();
    assert_present_rows(initial_value, &initial_indices, &entities);
    assert!(initial_value.continuation().is_some());
    assert_initial_window_work(&initial, VISIBLE_ROWS);

    let tail_index = row_count - 1;
    super::super::update_projection_identity(
        &mut workspace,
        entities[tail_index].clone(),
        "receipt-order--tail",
    );
    let refresh = match workspace
        .with_execution_request_mut(|execution, mut workspace| {
            live.refresh(execution, &mut workspace)
                .expect("QP04 Query refresh")
        })
        .expect("workspace host call opens")
    {
        UiCollectionProjectionRefreshOutcome::Applied(receipt) => receipt,
        UiCollectionProjectionRefreshOutcome::NoSemanticDelivery => {
            panic!("tail reorder must change the visible window")
        }
    };
    let refreshed = present(refresh.fact());
    assert_present_rows(refreshed, &[tail_index], &entities);
    assert!(refreshed.continuation().is_some());
    assert!(refresh.fact().changes().iter().any(|change| matches!(
        change,
        UiCollectionProjectionChange::Insert { row, at: 0 }
            if row.query_identity() == &entities[tail_index].evidence_identity()
    )));
    assert!(refresh.fact().changes().iter().any(|change| matches!(
        change,
        UiCollectionProjectionChange::Remove { row, from }
            if *from == VISIBLE_ROWS - 1
                && row.query_identity() == &entities[VISIBLE_ROWS - 1].evidence_identity()
    )));
    close_projection(live, &mut workspace);

    let (reopened, post_maintenance) = open_projection(&mut workspace, VISIBLE_ROWS as u32);
    let post_maintenance_value = present(&post_maintenance);
    let post_maintenance_indices = std::iter::once(tail_index)
        .chain(0..VISIBLE_ROWS - 1)
        .collect::<Vec<_>>();
    assert_present_rows(post_maintenance_value, &post_maintenance_indices, &entities);
    assert!(post_maintenance_value.continuation().is_some());
    assert_initial_window_work(&post_maintenance, VISIBLE_ROWS);
    close_projection(reopened, &mut workspace);
}

fn authored_rows(row_count: usize) -> Vec<(String, String)> {
    (0..row_count)
        .map(|index| {
            (
                format!("receipt-order.{index:05}"),
                format!("Value {index:05}"),
            )
        })
        .collect()
}

fn open_projection(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    max_rows: u32,
) -> (
    UiLiveCollectionProjection,
    UiCollectionProjectionFactReceipt,
) {
    let installed = workspace.worth_ui().expect("WORTH UI domain installed");
    let registration = UiCollectionProjectionRegistration::text(
        installed
            .projection_view("certification.collection.qp04")
            .expect("QP04 installed collection view"),
        UiProjectionFieldRequirement::declared("identity.id").expect("row identity field"),
        [UiProjectionFieldRequirement::declared("status").expect("selected field")],
        false,
        true,
    )
    .expect("QP04 collection registration");
    let UiCollectionProjectionBindingAdmission::Ready(binding) = registration.admit(workspace)
    else {
        panic!("QP04 binding must admit");
    };
    let budget = UiCollectionProjectionBudget::new(max_rows, 131_072, 1, 8_388_608)
        .expect("QP04 collection budget");
    let UiCollectionProjectionOpenOutcome::Opened(opened) = workspace
        .with_execution_request_mut(|execution, workspace| {
            binding.open(execution, budget, workspace)
        })
        .expect("workspace host call opens")
    else {
        panic!("QP04 collection projection must open");
    };
    opened.into_parts()
}

fn present(fact: &UiCollectionProjectionFactReceipt) -> &UiCollectionProjectionValue {
    match fact.availability() {
        UiProjectionAvailability::Present(UiPresentProjection::Current(value)) => value,
        other => panic!("bounded seed did not produce current collection truth: {other:?}"),
    }
}

fn assert_present_rows(
    value: &UiCollectionProjectionValue,
    expected_indices: &[usize],
    entities: &[worth_query::facade::foundation::WorthQueryEntityIdentity],
) {
    assert_eq!(value.rows().len(), expected_indices.len());
    for (row, index) in value.rows().iter().zip(expected_indices) {
        assert_eq!(row.selected_values().len(), 1);
        assert_eq!(
            row.selected_values()[0].as_str(),
            format!("Value {index:05}")
        );
        assert_eq!(
            row.row().query_identity(),
            &entities[*index].evidence_identity()
        );
    }
}

fn assert_initial_window_work(fact: &UiCollectionProjectionFactReceipt, row_count: usize) {
    assert_eq!(fact.work().rows_visited(), row_count);
    assert_eq!(fact.work().selected_key_accesses(), row_count);
    assert_eq!(fact.work().indexed_row_lookups(), row_count);
    assert_eq!(fact.work().native_values_materialized(), row_count);
    assert_eq!(fact.work().continuation_operations(), 1);
    assert_eq!(fact.work().unrelated_width_scans(), 0);
    assert_eq!(fact.work().key_resolution_key_scans(), 0);
}

fn close_projection(
    live: UiLiveCollectionProjection,
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) {
    let UiLiveCollectionProjectionCloseOutcome::Closed(closed) = live.close(workspace) else {
        panic!("QP04 live collection owner must close");
    };
    assert!(closed.owner_terminal());
}
