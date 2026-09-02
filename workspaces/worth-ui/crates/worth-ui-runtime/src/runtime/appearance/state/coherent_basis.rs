use std::num::NonZeroU64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceTarget {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    component_reference: Option<crate::capability::ComponentId>,
    selection_key: Option<NonZeroU64>,
}

impl UiAppearanceTarget {
    pub(crate) fn new(
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> Result<Self, UiAppearanceTargetDenial> {
        if node_receipt.mounted_instance() != mounted_instance {
            return Err(UiAppearanceTargetDenial::ReceiptInstanceMismatch);
        }
        Ok(Self {
            session,
            surface,
            graph_node,
            mounted_instance,
            incarnation,
            node_receipt,
            component_reference: None,
            selection_key: None,
        })
    }

    pub(crate) const fn with_selection_key(mut self, key: NonZeroU64) -> Self {
        self.selection_key = Some(key);
        self
    }

    pub(crate) fn with_component_reference(
        mut self,
        component: crate::capability::ComponentId,
    ) -> Self {
        self.component_reference = Some(component);
        self
    }

    pub(crate) const fn session(&self) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session
    }
    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }
    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }
    pub(crate) const fn mounted_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }
    pub(crate) const fn incarnation(&self) -> worth_ui_host_contract::UiMountIncarnation {
        self.incarnation
    }
    pub(crate) const fn node_receipt(
        &self,
    ) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub(crate) const fn selection_key(&self) -> Option<NonZeroU64> {
        self.selection_key
    }

    pub(crate) fn component_reference(&self) -> Option<&crate::capability::ComponentId> {
        self.component_reference.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceTargetDenial {
    ReceiptInstanceMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceCoherentBasis {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    turn: crate::runtime::observation::UiObservationTurnIdentity,
    source_basis: u64,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph_node: crate::graph::UiGraphNodeIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    owner_revisions: [u64; 6],
}

impl UiAppearanceCoherentBasis {
    pub(crate) fn seal(
        snapshot: &super::UiAppearanceOwnerSnapshot,
        target: &UiAppearanceTarget,
        owner_revisions: [u64; 6],
    ) -> Self {
        Self {
            session: snapshot.session(),
            generation: snapshot.generation().clone(),
            turn: snapshot.turn(),
            source_basis: snapshot.source_basis(),
            surface: target.surface(),
            graph_node: target.graph_node(),
            mounted_instance: target.mounted_instance(),
            incarnation: target.incarnation(),
            node_receipt: target.node_receipt(),
            owner_revisions,
        }
    }

    pub(crate) const fn session(&self) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session
    }
    pub(crate) const fn generation(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }
    pub(crate) const fn turn(&self) -> crate::runtime::observation::UiObservationTurnIdentity {
        self.turn
    }
    pub(crate) const fn source_basis(&self) -> u64 {
        self.source_basis
    }
    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }
    pub(crate) const fn graph_node(&self) -> crate::graph::UiGraphNodeIdentity {
        self.graph_node
    }
    pub(crate) const fn mounted_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }
    pub(crate) const fn incarnation(&self) -> worth_ui_host_contract::UiMountIncarnation {
        self.incarnation
    }
    pub(crate) const fn node_receipt(
        &self,
    ) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub(crate) const fn owner_revisions(&self) -> &[u64; 6] {
        &self.owner_revisions
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325_u64;
        digest = fold(digest, self.session.as_u64());
        digest = fold(digest, self.generation.session_identity().as_u64());
        digest = fold(
            digest,
            self.generation
                .prepared_generation()
                .semantic_package_identity()
                .narrowing_fingerprint(),
        );
        digest = fold(digest, self.surface.diagnostic_value());
        digest = fold(digest, self.graph_node.digest());
        digest = fold(digest, self.mounted_instance.diagnostic_value());
        digest = fold(digest, self.incarnation.diagnostic_value());
        fold(digest, self.node_receipt.diagnostic_value())
    }

    pub(crate) fn evidence_digest(&self) -> u64 {
        let mut digest = self.semantic_digest();
        digest = fold(digest, self.turn.as_u64());
        digest = fold(digest, self.source_basis);
        for revision in self.owner_revisions {
            digest = fold(digest, revision);
        }
        digest
    }
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}
