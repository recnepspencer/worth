pub(super) fn settle_published_portal_proposal(
    admitted: &mut super::UiIntentConsequenceAdmitted<'_>,
    mounted: &crate::mounting::UiMountedFramePublicationReceipt,
) -> Option<crate::facade::entry::focus_placement::UiSemanticFocusPublicationReceipt> {
    let transaction = admitted.transfer.portal_proposal.take()?;
    let binding_commit = transaction.overlay_binding_commit();
    let (focus, motion, exit_retention) = admitted
        .session
        .application
        .settle_published_portal_service_proposal(
            transaction,
            mounted,
            admitted
                .session
                .portal
                .as_mut()
                .expect("staged proposal retains Portal installation"),
            admitted
                .session
                .focus
                .as_mut()
                .expect("staged proposal retains Focus installation"),
            admitted.session.scroll.as_mut(),
            admitted.session.selection.as_mut(),
            admitted
                .session
                .motion
                .as_mut()
                .expect("staged proposal retains Motion installation"),
        )
        .expect("exact staged portal proposal accepts its publication receipt");
    let binding_commit = binding_commit.with_retained_exit(exit_retention.is_some());
    admitted
        .session
        .commit_authored_overlay_binding(binding_commit)
        .expect("published Portal transition retains its exact overlay binding stage");
    admitted
        .session
        .rebind_portal_after_current_published_frame();
    let focus = admitted
        .session
        .place_committed_semantic_focus(focus, mounted)
        .expect("published Focus successor retains exact mounted presentation basis");
    admitted
        .session
        .install_portal_exit_retention(exit_retention);
    admitted.session.install_committed_motion(motion);
    Some(focus)
}

pub(super) fn settle_indeterminate_portal_proposal(
    admitted: &mut super::UiIntentConsequenceAdmitted<'_>,
) -> Option<crate::runtime::session::UiIndeterminatePortalProposalTransaction> {
    let transaction = admitted.transfer.portal_proposal.take()?;
    Some(
        admitted
            .session
            .application
            .settle_indeterminate_portal_service_proposal(
                transaction,
                admitted
                    .session
                    .portal
                    .as_mut()
                    .expect("staged proposal retains Portal installation"),
                admitted
                    .session
                    .focus
                    .as_mut()
                    .expect("staged proposal retains Focus installation"),
                admitted
                    .session
                    .motion
                    .as_mut()
                    .expect("staged proposal retains Motion installation"),
            )
            .expect("indeterminate physical settlement retains exact semantic proposal"),
    )
}

pub(super) fn settle_rejected_portal_proposal(
    admitted: &mut super::UiIntentConsequenceAdmitted<'_>,
) {
    let Some(transaction) = admitted.transfer.portal_proposal.take() else {
        return;
    };
    admitted
        .session
        .application
        .settle_rejected_portal_service_proposal(
            transaction,
            admitted
                .session
                .focus
                .as_mut()
                .expect("staged proposal retains Focus installation"),
            admitted
                .session
                .motion
                .as_mut()
                .expect("staged proposal retains Motion installation"),
        )
        .expect("before-effect rejection retains exact semantic proposal");
}
