#[derive(Clone, Copy)]
pub(super) struct ContextIdentities {
    pub(super) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(super) issuer: worth_ui_host_contract::UiMountedNodeReceiptIssuer,
    pub(super) instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(super) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(super) incarnation: worth_ui_host_contract::UiMountIncarnation,
}

pub(super) fn context_identities() -> ContextIdentities {
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    ContextIdentities {
        frame,
        issuer,
        instance,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        incarnation: worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap(),
    }
}

pub(super) fn context(
    identities: ContextIdentities,
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ordinal: u64,
) -> crate::runtime::appearance::UiAppearanceAttemptContext {
    let target = crate::runtime::appearance::UiAppearanceTarget::new(
        session,
        identities.surface,
        crate::graph::UiGraphNodeIdentity::new(ordinal),
        identities.instance,
        identities.incarnation,
        identities.issuer.receipt_for(identities.instance),
    )
    .unwrap();
    crate::runtime::appearance::UiAppearanceAttemptContext::new(
        target,
        identities.frame,
        identities.issuer,
        ordinal,
        worth_ui_host_contract::UiMountedAllocationProjection::Omitted(
            worth_ui_host_contract::UiMountedOmissionReason::NoCommittedAllocation,
        ),
        crate::mounting::projection::appearance::UiMountedAppearanceClip::Unclipped,
        Some(0),
        None,
        Box::new([]),
        generation.clone(),
        0,
        0,
    )
}
