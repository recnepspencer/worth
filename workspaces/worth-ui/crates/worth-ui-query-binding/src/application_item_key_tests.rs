use core::num::NonZeroU64;

use crate::{
    scalar_text_projection_fixture::seeded_collection_projection_workspace_with_item_keys,
    UiCollectionProjectionBindingAdmission, UiCollectionProjectionBudget,
    UiCollectionProjectionOpenOutcome, UiPresentProjection, UiProjectionAvailability,
    UiProjectionFactStopKind, UiProjectionFieldRequirement, WorthUiQueryWorkspaceExt,
};

#[test]
fn collection_fact_materializes_the_declared_unsigned_application_item_key() {
    let (mut workspace, _) = seeded_collection_projection_workspace_with_item_keys(
        vec![("pulse.alpha".into(), "Alpha".into(), 315_051)],
        false,
        true,
        true,
    );
    let fact = open_with_application_item_key(&mut workspace);
    let UiProjectionAvailability::Present(UiPresentProjection::Current(value)) =
        fact.availability()
    else {
        panic!("typed application item key must materialize: {fact:?}")
    };

    assert_eq!(
        value.rows()[0].application_item_key(),
        NonZeroU64::new(315_051)
    );
}

#[test]
fn zero_application_item_key_stops_collection_fact_materialization() {
    let (mut workspace, _) = seeded_collection_projection_workspace_with_item_keys(
        vec![("pulse.alpha".into(), "Alpha".into(), 0)],
        false,
        true,
        true,
    );
    let fact = open_with_application_item_key(&mut workspace);

    assert!(matches!(
        fact.availability(),
        UiProjectionAvailability::Stopped(stop)
            if stop.kind() == UiProjectionFactStopKind::PayloadShapeMismatch
    ));
}

fn open_with_application_item_key(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
) -> crate::UiCollectionProjectionFactReceipt {
    let installed = workspace.worth_ui().expect("Worth UI domain installed");
    let registration =
        crate::UiCollectionProjectionRegistration::text(
            installed
                .projection_view("support.collection.status")
                .expect("installed collection view"),
            UiProjectionFieldRequirement::identity_id(),
            [UiProjectionFieldRequirement::collection_item_status()],
            false,
            true,
        )
        .expect("collection registration")
        .with_unsigned64_application_item_key_field(
            UiProjectionFieldRequirement::collection_item_key(),
        );
    let binding = match registration.admit(workspace) {
        UiCollectionProjectionBindingAdmission::Ready(binding) => binding,
        admission => panic!("typed collection registration must admit: {admission:?}"),
    };
    let budget =
        UiCollectionProjectionBudget::new(1, 1, 0, 1_024).expect("single-row collection budget");
    let UiCollectionProjectionOpenOutcome::Opened(opened) = binding.open(budget, workspace) else {
        panic!("typed collection projection must reach fact materialization")
    };
    opened.into_parts().1
}
