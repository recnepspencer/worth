use worth_ui_host_contract::UiHostObservationPresentationBasis;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPresentedTargetFrameRelation {
    Current,
    Retained,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UiPresentedInteractionTarget {
    presentation: UiHostObservationPresentationBasis,
    relation: UiPresentedTargetFrameRelation,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    hit_test_order: u32,
    semantic_digest: u64,
    hit_test_rows_considered: usize,
    geometry: super::UiPresentedInteractionGeometry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPresentedInteractionTargetView {
    presentation: UiHostObservationPresentationBasis,
    relation: UiPresentedTargetFrameRelation,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    hit_test_order: u32,
    semantic_digest: u64,
    hit_test_rows_considered: usize,
    geometry: super::UiPresentedInteractionGeometry,
}

pub(super) fn seal_target(
    presentation: UiHostObservationPresentationBasis,
    relation: UiPresentedTargetFrameRelation,
    current: crate::mounting::UiCurrentHitTarget,
    presented: crate::mounting::UiPresentedHitTestRow,
    hit_test_rows_considered: usize,
) -> UiPresentedInteractionTarget {
    let row = current.row();
    UiPresentedInteractionTarget {
        presentation,
        relation,
        surface: row.surface(),
        binding: row.binding(),
        mounted_instance: row.mounted_instance(),
        node_receipt: row.node_receipt(),
        hit_test_order: row.order().rank(),
        semantic_digest: row.semantic_digest(),
        hit_test_rows_considered,
        geometry: super::UiPresentedInteractionGeometry::from_presented_hit_test(
            presented,
            presentation,
        ),
    }
}

impl UiPresentedInteractionTarget {
    pub const fn view(&self) -> UiPresentedInteractionTargetView {
        UiPresentedInteractionTargetView {
            presentation: self.presentation,
            relation: self.relation,
            surface: self.surface,
            binding: self.binding,
            mounted_instance: self.mounted_instance,
            node_receipt: self.node_receipt,
            hit_test_order: self.hit_test_order,
            semantic_digest: self.semantic_digest,
            hit_test_rows_considered: self.hit_test_rows_considered,
            geometry: self.geometry,
        }
    }

    pub const fn presentation(&self) -> UiHostObservationPresentationBasis {
        self.presentation
    }
    pub const fn frame_relation(&self) -> UiPresentedTargetFrameRelation {
        self.relation
    }
    pub const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }
    pub const fn binding(&self) -> worth_ui_host_contract::UiSurfaceBindingGeneration {
        self.binding
    }
    pub const fn mounted_instance(&self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }
    pub const fn node_receipt(&self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub const fn hit_test_order(&self) -> u32 {
        self.hit_test_order
    }
    pub const fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }
    pub const fn hit_test_rows_considered(&self) -> usize {
        self.hit_test_rows_considered
    }
}

impl UiPresentedInteractionTargetView {
    pub const fn presentation(self) -> UiHostObservationPresentationBasis {
        self.presentation
    }
    pub const fn frame_relation(self) -> UiPresentedTargetFrameRelation {
        self.relation
    }
    pub const fn surface(self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }
    pub const fn binding(self) -> worth_ui_host_contract::UiSurfaceBindingGeneration {
        self.binding
    }
    pub const fn mounted_instance(self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }
    pub const fn node_receipt(self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub const fn hit_test_order(self) -> u32 {
        self.hit_test_order
    }
    pub const fn semantic_digest(self) -> u64 {
        self.semantic_digest
    }
    pub const fn hit_test_rows_considered(self) -> usize {
        self.hit_test_rows_considered
    }
    pub(crate) const fn geometry(self) -> super::UiPresentedInteractionGeometry {
        self.geometry
    }

    /// The same target after its instance's receipt succeeded. The
    /// presentation it was observed in stays historical evidence.
    pub(crate) const fn with_node_receipt(
        self,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> Self {
        Self {
            node_receipt,
            ..self
        }
    }

    /// The same target after its surface was rebound. The presentation it
    /// was observed in stays historical evidence.
    pub(crate) const fn with_binding(
        self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Self {
        Self { binding, ..self }
    }
}

#[cfg(test)]
pub(crate) fn interaction_target_view_for_test(
    presentation: UiHostObservationPresentationBasis,
    affinity: crate::mounting::UiMountedInteractionAffinityInput,
) -> UiPresentedInteractionTargetView {
    UiPresentedInteractionTargetView {
        presentation,
        relation: UiPresentedTargetFrameRelation::Current,
        surface: affinity.surface,
        binding: affinity.binding,
        mounted_instance: affinity.mounted_instance,
        node_receipt: affinity.node_receipt,
        hit_test_order: 0,
        semantic_digest: 1,
        hit_test_rows_considered: 1,
        geometry: super::UiPresentedInteractionGeometry::for_test(presentation),
    }
}
