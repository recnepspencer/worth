use super::{rebind_portal_after_published_frame, UiMountedPublicationSettlementPorts};

pub(super) fn reconcile_focus_after_published_frame_with_ports(
    ports: &mut UiMountedPublicationSettlementPorts<'_>,
    publication: &crate::mounting::UiMountedFramePublicationReceipt,
) {
    if let Some(portal) = ports.portal.as_deref_mut() {
        rebind_portal_after_published_frame(portal, publication);
    }
    let Some(focus) = ports.focus.as_deref_mut() else {
        return;
    };
    let Some(snapshot) = ports.mounted.focus_participation_snapshot() else {
        return;
    };
    let transition = focus
        .reconcile_mounted_participation(&snapshot)
        .expect("mounted participant bounds fit the focus owner counters")
        .transition();
    let Some(transition) = transition else {
        return;
    };
    place_reconciled_focus(ports, Some(transition), publication);
}

pub(super) fn place_reconciled_focus(
    ports: &mut UiMountedPublicationSettlementPorts<'_>,
    transition: Option<crate::runtime::focus::UiFocusTransitionReceipt>,
    publication: &crate::mounting::UiMountedFramePublicationReceipt,
) {
    let Some(transition) = transition else {
        return;
    };
    super::super::focus_placement::ports::UiFocusPlacementPorts::new(
        ports.mounted,
        ports
            .focus
            .as_deref_mut()
            .expect("focus placement requires installed Focus support"),
        ports.interaction,
        ports.host_session,
        ports.active_generation.clone(),
    )
    .place(transition, publication)
    .expect("reconciled Focus successor retains exact mounted presentation basis");
}
