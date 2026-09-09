use worth_ui_host_contract::{
    UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFragmentIdentity,
};

pub(super) fn capture(
    session: &crate::facade::WorthUiActiveApplicationSession,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
) -> UiUnpublishedAppearanceFragment {
    session.mounted.current_unpublished_appearance().unwrap().unwrap().fragments().iter()
        .find(|fragment| matches!(fragment.identity(), UiUnpublishedAppearanceFragmentIdentity::NodeReceipt { successor: Some(receipt), .. } if receipt.mounted_instance() == instance))
        .expect("the preceding theme update rendered the retiring instance").clone()
}

pub(super) fn verify_retry(
    session: &crate::facade::WorthUiActiveApplicationSession,
    frame: &crate::mounting::UiPreparedMountedFrame,
) {
    let receipt = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap()
        .fragments()
        .iter()
        .find_map(|fragment| match fragment.identity() {
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                successor: Some(receipt),
                ..
            } if session
                .mounted
                .current_mounted_identity_basis(receipt.mounted_instance())
                .is_none() =>
            {
                Some(receipt)
            }
            _ => None,
        })
        .expect("real unmount retired a previously rendered instance");
    frame.verify_unpublished_appearance_retirement_denial_and_retry(receipt);
}

pub(super) fn assert_removed(
    session: &crate::facade::WorthUiActiveApplicationSession,
    previous: &UiUnpublishedAppearanceFragment,
) {
    let UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
        successor: Some(receipt),
        ..
    } = previous.identity()
    else {
        panic!("the predecessor has a real mounted receipt");
    };
    let output = session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let removal = output
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity()
                == UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                    predecessor: Some(receipt),
                    successor: None,
                }
        })
        .expect("unmount emits its own removal beside changed survivors");
    assert_eq!(
        removal.work().posture(),
        worth_ui_host_contract::UiMountedAppearanceWorkPosture::Delta
    );
    assert_eq!(
        removal.work().changes(),
        &[
            worth_ui_host_contract::UiMountedAppearanceMechanicChange::Remove(
                worth_ui_host_contract::UiMountedAppearanceMechanicIdentity::Surface(
                    receipt.mounted_instance()
                ),
            )
        ]
    );
    let worth_ui_host_contract::UiMountedAppearanceMechanic::Surface(surface) =
        &previous.work().successor().mechanics()[0]
    else {
        panic!("the locality fixture renders a surface fill");
    };
    let bounds = surface.visual_bounds();
    assert_eq!(removal.work().damage().len(), 1);
    let damage = &removal.work().damage()[0];
    assert_eq!(
        (damage.x(), damage.y(), damage.width(), damage.height()),
        (bounds.x(), bounds.y(), bounds.width(), bounds.height())
    );
    worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
}
