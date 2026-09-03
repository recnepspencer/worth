use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiMountIncarnation, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiSemanticSurfaceIdentity,
};

impl super::UiAppearanceCoherentBasis {
    #[allow(
        clippy::too_many_arguments,
        reason = "test basis carries the exact identity fields consumed by direct adapters"
    )]
    pub(crate) fn for_test(
        snapshot: &super::super::UiAppearanceOwnerSnapshot,
        consumer: super::super::UiAppearanceStateConsumer,
        mounted_instance: UiMountedInstanceIdentity,
        incarnation: UiMountIncarnation,
        node_receipt: UiMountedNodeReceiptIdentity,
        surface: UiSemanticSurfaceIdentity,
        presentation: Option<UiHostObservationPresentationBasis>,
        selection: Option<super::super::UiAppearanceSelectionSelector>,
        operability_route: Option<Box<str>>,
    ) -> Self {
        Self {
            turn: snapshot.turn(),
            session: snapshot.session(),
            source_basis: snapshot.source_basis(),
            generation: snapshot.generation().clone(),
            graph_node: consumer.graph_node(),
            consumer,
            mounted_instance,
            incarnation,
            owner_node_receipt: node_receipt,
            node_receipt,
            surface,
            theme: crate::runtime::appearance::UiActiveThemeBinding::for_test(
                surface,
                snapshot.generation().clone(),
            ),
            presentation,
            selection,
            operability_route,
            owner_revisions: super::owner_revisions(snapshot),
        }
    }
}

pub(crate) fn validate_presentation_for_test(
    mounted: &crate::mounting::WorthUiMountedSessionState,
    presentation: Option<UiHostObservationPresentationBasis>,
) -> Result<(), super::UiAppearanceCoherentBasisDenial> {
    super::validate_presentation(mounted, presentation)
}
