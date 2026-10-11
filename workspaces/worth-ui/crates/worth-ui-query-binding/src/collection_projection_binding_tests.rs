use crate::{
    scalar_text_projection_fixture::{
        collection_projection_workspace, collection_projection_workspace_without_dependency_impact,
        insert_collection_status,
    },
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionOpenOutcome, UiLiveCollectionProjectionCloseOutcome, UiPresentProjection,
    UiProjectionAvailability, UiProjectionBindingStopKind, UiProjectionFactStopKind,
    WorthUiQueryWorkspaceExt,
};

#[test]
fn registered_collection_emits_affine_native_text_fact_and_closes_lease() {
    let mut workspace = collection_projection_workspace();
    let alpha = insert_collection_status(&mut workspace, "pulse.alpha", "Alpha");
    let bravo = insert_collection_status(&mut workspace, "pulse.bravo", "Bravo");
    let binding = admitted_binding(&workspace, "identity.id", "status", false, true);
    let UiCollectionProjectionOpenOutcome::Opened(opened) =
        binding.open(collection_budget(8, 1), &mut workspace)
    else {
        panic!("the exact collection registration must open");
    };
    let (live, fact) = opened.into_parts();
    let UiProjectionAvailability::Present(UiPresentProjection::Current(value)) =
        fact.availability()
    else {
        panic!("ready Query collection must produce a current fact");
    };
    let expected_identities = [alpha.evidence_identity(), bravo.evidence_identity()];
    let actual_identities = value
        .rows()
        .iter()
        .map(|row| row.row().query_identity().clone())
        .collect::<Vec<_>>();
    let actual_values = value
        .rows()
        .iter()
        .map(|row| row.selected_values()[0].as_str())
        .collect::<Vec<_>>();

    assert_eq!(actual_identities, expected_identities);
    assert_eq!(actual_values, ["Alpha", "Bravo"]);
    assert!(value.continuation().is_none());
    assert_eq!(fact.work().rows_visited(), 2);
    assert_eq!(fact.work().selected_key_accesses(), 2);
    assert_eq!(fact.work().indexed_row_lookups(), 2);
    assert_eq!(fact.work().native_values_materialized(), 2);
    assert_eq!(fact.work().unrelated_width_scans(), 0);
    assert_eq!(fact.work().key_resolution_key_scans(), 0);
    assert!(live.is_current_installation());
    let UiLiveCollectionProjectionCloseOutcome::Closed(closed) = live.close(&mut workspace) else {
        panic!("owning workspace must close the exact collection lease");
    };
    assert!(closed.owner_terminal());
    assert_eq!(closed.counters().close_attempts, 1);
    assert_eq!(closed.counters().unrelated_owner_scans, 0);
}

#[test]
fn continuation_and_requirement_stops_are_explicit_facts() {
    let mut workspace = collection_projection_workspace();
    insert_collection_status(&mut workspace, "pulse.alpha", "Alpha");
    insert_collection_status(&mut workspace, "pulse.bravo", "Bravo");
    let binding = admitted_binding(&workspace, "identity.id", "status", false, true);
    let UiCollectionProjectionOpenOutcome::Opened(opened) =
        binding.open(collection_budget(1, 1), &mut workspace)
    else {
        panic!("continuation-capable binding must open");
    };
    let (live, fact) = opened.into_parts();
    let UiProjectionAvailability::Present(UiPresentProjection::Current(value)) =
        fact.availability()
    else {
        panic!("bounded collection must remain present");
    };
    assert_eq!(value.rows().len(), 1);
    assert!(value.continuation().is_some());
    assert_eq!(fact.work().continuation_operations(), 1);
    assert!(matches!(
        live.close(&mut workspace),
        UiLiveCollectionProjectionCloseOutcome::Closed(_)
    ));

    let binding = admitted_binding(&workspace, "identity.id", "status", false, false);
    let UiCollectionProjectionOpenOutcome::Opened(opened) =
        binding.open(collection_budget(1, 1), &mut workspace)
    else {
        panic!("continuation policy is represented by fact posture");
    };
    let (live, fact) = opened.into_parts();
    let UiProjectionAvailability::Stopped(stop) = fact.availability() else {
        panic!("forbidden continuation must stop the fact");
    };
    assert_eq!(stop.kind(), UiProjectionFactStopKind::PayloadShapeMismatch);
    assert!(matches!(
        live.close(&mut workspace),
        UiLiveCollectionProjectionCloseOutcome::Closed(_)
    ));
}

#[test]
fn collection_registration_rejects_row_schema_and_foreign_world_twins() {
    let owner = collection_projection_workspace();
    let foreign = collection_projection_workspace();
    let wrong_row = registration(&owner, "identity.other", "status", false, true);
    let wrong_field = registration(&owner, "identity.id", "missing", false, true);
    let owner_registration = registration(&owner, "identity.id", "status", false, true);

    assert_stopped(
        wrong_row.admit(&owner),
        UiProjectionBindingStopKind::RowIdentityMismatch,
    );
    assert_stopped(
        wrong_field.admit(&owner),
        UiProjectionBindingStopKind::SchemaMismatch,
    );
    assert_stopped(
        owner_registration.admit(&foreign),
        UiProjectionBindingStopKind::WrongWorld,
    );
}

#[test]
fn collection_registration_rejects_a_host_missing_live_dependency_impact() {
    let workspace = collection_projection_workspace_without_dependency_impact();
    assert_stopped(
        registration(&workspace, "identity.id", "status", false, true).admit(&workspace),
        UiProjectionBindingStopKind::LifecycleMismatch,
    );
}

fn registration(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
    row: &str,
    field: &str,
    requires_complete: bool,
    permits_continuation: bool,
) -> crate::UiCollectionProjectionRegistration {
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    crate::UiCollectionProjectionRegistration::text(
        installed
            .projection_view("support.collection.status")
            .expect("valid projection view identity"),
        crate::UiProjectionFieldRequirement::declared(row).expect("valid row field"),
        [crate::UiProjectionFieldRequirement::declared(field).expect("valid selected field")],
        requires_complete,
        permits_continuation,
    )
    .expect("canonical collection registration")
}

fn admitted_binding(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
    row: &str,
    field: &str,
    requires_complete: bool,
    permits_continuation: bool,
) -> crate::UiCollectionProjectionBinding {
    match registration(
        workspace,
        row,
        field,
        requires_complete,
        permits_continuation,
    )
    .admit(workspace)
    {
        UiCollectionProjectionBindingAdmission::Ready(binding) => binding,
        UiCollectionProjectionBindingAdmission::Stopped(stop) => {
            panic!("exact registration stopped: {stop:?}")
        }
    }
}

fn assert_stopped(
    admission: UiCollectionProjectionBindingAdmission,
    expected: UiProjectionBindingStopKind,
) {
    let UiCollectionProjectionBindingAdmission::Stopped(stop) = admission else {
        panic!("hostile registration must stop");
    };
    assert_eq!(stop.kind(), expected);
}

fn collection_budget(rows: u32, continuations: usize) -> UiCollectionProjectionBudget {
    UiCollectionProjectionBudget::new(rows, 64, continuations, 65_536)
        .expect("test collection budget")
}
