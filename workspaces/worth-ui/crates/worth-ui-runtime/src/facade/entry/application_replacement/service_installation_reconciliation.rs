pub(super) fn reconcile_focus_installation(
    slot: &mut crate::runtime::UiRuntimeServiceInstallation<
        crate::runtime::focus::UiFocusRuntimeState,
    >,
    policy: Option<crate::declaration::UiFocusPolicy>,
) {
    let owner = if let Some(policy) = policy {
        let mut owner = slot.take().unwrap_or_else(|| {
            crate::runtime::focus::UiFocusRuntimeState::new_session_restore_candidate_with_policy(
                policy,
            )
        });
        owner.apply_policy(policy);
        Some(owner)
    } else {
        if let Some(mut owner) = slot.take() {
            let _released = owner.shutdown();
        }
        None
    };
    *slot = crate::runtime::UiRuntimeServiceInstallation::from_optional(owner);
}

pub(super) fn reconcile_portal_installation(
    slot: &mut crate::runtime::UiRuntimeServiceInstallation<
        crate::runtime::portal::UiPortalRuntimeState,
    >,
    policy: Option<crate::declaration::UiPortalPolicy>,
    dormant_ordinal_issuer: &mut Option<crate::runtime::portal::UiPortalStackOrdinalIssuer>,
) {
    let owner = if let Some(policy) = policy {
        let mut owner = slot.take().unwrap_or_else(|| {
            crate::runtime::portal::UiPortalRuntimeState::new_with_policy_and_ordinal_issuer(
                crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate,
                policy,
                dormant_ordinal_issuer
                    .take()
                    .expect("active session retains one Portal ordinal issuer"),
            )
        });
        owner.apply_policy(policy);
        Some(owner)
    } else {
        if let Some(mut owner) = slot.take() {
            debug_assert_eq!(owner.shutdown().final_active_records(), 0);
            let issuer = owner.take_stack_ordinal_issuer();
            debug_assert!(dormant_ordinal_issuer.replace(issuer).is_none());
        }
        None
    };
    *slot = crate::runtime::UiRuntimeServiceInstallation::from_optional(owner);
}

pub(super) fn reconcile_motion_installation(
    slot: &mut crate::runtime::UiRuntimeServiceInstallation<
        crate::runtime::motion::UiMotionRuntimeState,
    >,
    policy: Option<crate::declaration::UiMotionPolicy>,
) {
    let owner = if let Some(policy) = policy {
        let mut owner = slot.take().unwrap_or_else(|| {
            crate::runtime::motion::UiMotionRuntimeState::new_with_policy(
                crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
                policy,
            )
        });
        owner.apply_policy(policy);
        Some(owner)
    } else {
        if let Some(mut owner) = slot.take() {
            debug_assert!(owner.shutdown().final_census().is_zero());
        }
        None
    };
    *slot = crate::runtime::UiRuntimeServiceInstallation::from_optional(owner);
}
