mod hit_test_presentation;
mod input_shielding;
pub(in crate::mounting) use input_shielding::UiModalInputAdmission;
mod text_paint;

#[derive(Clone)]
pub(crate) struct UiMountedVisualRegionBasis {
    hit_test: crate::mounting::projection::UiMountedHitMechanicSource,
    pub(in crate::mounting) presented_hits:
        crate::mounting::presented_hit_index::UiPresentedHitIndex,
    semantic_text: crate::mounting::projection::UiMountedSemanticMechanicSource,
    portal_overlays: std::rc::Rc<[worth_ui_host_contract::UiMountedPortalOverlayMechanic]>,
    portal_input_order:
        std::rc::Rc<std::collections::BTreeMap<u64, crate::runtime::portal::UiPortalStackOrdinal>>,
    /// Where the frame presents each Portal child it does not present in
    /// place.
    portal_children: std::rc::Rc<
        std::collections::BTreeMap<
            worth_ui_host_contract::UiMountedInstanceIdentity,
            super::UiMountedPlacement,
        >,
    >,
    text_clips: std::rc::Rc<
        std::collections::BTreeMap<
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::mounting::projection::UiMountedAppearanceClip,
        >,
    >,
    appearance_paint: std::sync::Arc<[super::UiMountedAppearancePaintBasis]>,
    indexed_motion_reserved_bytes: Option<usize>,
    direct_scroll_reserved_bytes: Option<usize>,
    binding: Option<worth_ui_host_contract::UiSurfaceBindingGeneration>,
    receipts: Option<super::UiMountedNodeReceiptBasis>,
    #[cfg(test)]
    materialized: Option<UiMaterializedVisualRegionBasis>,
}

pub(crate) struct UiMountedHitTestPresentation {
    mechanic: worth_ui_host_contract::UiMountedHitTestMechanic,
    portal: Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
    owns_presented_portal: bool,
    ancestor_clip: crate::mounting::UiHitAncestorClip,
}

#[cfg(test)]
#[derive(Clone)]
struct UiMaterializedVisualRegionBasis {
    hit_test: std::sync::Arc<[worth_ui_host_contract::UiMountedHitTestMechanic]>,
    unsupported_paint: std::sync::Arc<[UiMountedUnsupportedPaintBasis]>,
}

#[derive(Clone, Copy)]
pub(crate) struct UiMountedUnsupportedPaintBasis {
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    bounds: worth_ui_host_contract::UiMountedCanonicalBox,
    clip: worth_ui_host_contract::UiMountedCanonicalBox,
    semantic_order: u32,
    source_digest: u64,
}

impl UiMountedVisualRegionBasis {
    pub(in crate::mounting) fn from_persistent(
        hit_test: crate::mounting::projection::UiMountedHitMechanicSource,
        semantic_text: crate::mounting::projection::UiMountedSemanticMechanicSource,
        presented_hits: crate::mounting::presented_hit_index::UiPresentedHitIndex,
    ) -> Self {
        Self {
            hit_test,
            presented_hits,
            semantic_text,
            portal_overlays: std::rc::Rc::from([]),
            portal_input_order: Default::default(),
            portal_children: Default::default(),
            text_clips: Default::default(),
            appearance_paint: std::sync::Arc::from([]),
            indexed_motion_reserved_bytes: Some(0),
            direct_scroll_reserved_bytes: Some(0),
            binding: None,
            receipts: None,
            #[cfg(test)]
            materialized: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn new(
        unsupported_paint: Box<[UiMountedUnsupportedPaintBasis]>,
        hit_test: Box<[worth_ui_host_contract::UiMountedHitTestMechanic]>,
    ) -> Self {
        Self {
            hit_test: Default::default(),
            presented_hits: Default::default(),
            semantic_text: Default::default(),
            portal_overlays: std::rc::Rc::from([]),
            portal_input_order: Default::default(),
            portal_children: Default::default(),
            text_clips: Default::default(),
            appearance_paint: std::sync::Arc::from([]),
            indexed_motion_reserved_bytes: Some(0),
            direct_scroll_reserved_bytes: Some(0),
            binding: None,
            receipts: None,
            materialized: Some(UiMaterializedVisualRegionBasis {
                hit_test: hit_test.into(),
                unsupported_paint: unsupported_paint.into(),
            }),
        }
    }

    pub(crate) fn for_binding(
        &self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        receipts: super::UiMountedNodeReceiptBasis,
    ) -> Self {
        Self {
            hit_test: self.hit_test.clone(),
            presented_hits: self.presented_hits.clone(),
            semantic_text: self.semantic_text.clone(),
            portal_overlays: std::rc::Rc::clone(&self.portal_overlays),
            portal_input_order: std::rc::Rc::clone(&self.portal_input_order),
            portal_children: std::rc::Rc::clone(&self.portal_children),
            text_clips: std::rc::Rc::clone(&self.text_clips),
            appearance_paint: std::sync::Arc::clone(&self.appearance_paint),
            indexed_motion_reserved_bytes: self.indexed_motion_reserved_bytes,
            direct_scroll_reserved_bytes: self.direct_scroll_reserved_bytes,
            binding: Some(binding),
            receipts: Some(receipts),
            #[cfg(test)]
            materialized: self.materialized.clone(),
        }
    }

    /// The overlay this basis presented for one Portal's Motion target.
    pub(crate) fn presented_portal_overlay(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic> {
        self.portal_overlays.iter().copied().find(|portal| {
            self.binding
                .is_none_or(|binding| portal.binding() == binding)
                && crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(
                    portal.surface(),
                    portal.owner(),
                    portal.portal_identity(),
                ) == target
        })
    }

    pub(crate) fn hit_test(&self) -> Box<[UiMountedHitTestPresentation]> {
        #[cfg(test)]
        if let Some(materialized) = &self.materialized {
            return materialized
                .hit_test
                .iter()
                .copied()
                .map(UiMountedHitTestPresentation::for_test)
                .collect();
        }
        let mut portal_owners = std::collections::BTreeSet::new();
        for portal in self.portal_overlays.iter().copied() {
            portal_owners.insert(portal.owner());
        }
        self.hit_test
            .iter()
            .map(|(_, row)| *row)
            .filter(|row| self.binding.is_none_or(|binding| row.binding() == binding))
            .filter_map(|row| {
                let row = self.receipts.as_ref().map_or(row, |receipts| {
                    crate::mounting::projection::reattribute_hit_test(
                        row,
                        receipts.frame(),
                        receipts,
                    )
                    .expect("retained hit rows belong to the presented receipt basis")
                });
                let placement = self.placement_of(row.mounted_instance());
                // The mechanic source holds each row where layout put it.
                let mechanic = placement
                    .present(super::UiLaidOut::from_layout(row))
                    .expect("validated Portal-relative hit region remains canonical")?
                    .into_shown();
                let portal = placement.portal();
                // The frame published each row with the ancestor clips it sits
                // inside; a row it did not publish is not presented.
                let ancestor_clip = self
                    .presented_hits
                    .published_ancestor_clip(mechanic.mounted_instance())?;
                Some(UiMountedHitTestPresentation {
                    mechanic,
                    portal,
                    owns_presented_portal: portal_owners.contains(&mechanic.mounted_instance()),
                    ancestor_clip,
                })
            })
            .collect()
    }

    pub(crate) fn appearance_paint(&self) -> Box<[super::UiMountedAppearancePaintBasis]> {
        self.appearance_paint
            .iter()
            .copied()
            .filter(|row| self.binding.is_none_or(|binding| row.binding() == binding))
            .filter_map(|row| {
                self.receipts
                    .as_ref()
                    .map_or(Some(row), |receipts| row.reattribute(receipts))
            })
            .collect()
    }

    pub(in crate::mounting) fn with_appearance_paint(
        mut self,
        mechanics: Vec<super::UiMountedRetainedAppearanceVisualMechanic>,
        bindings: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiSurfaceBindingGeneration,
        )],
    ) -> Self {
        self.appearance_paint = mechanics
            .into_iter()
            .filter_map(|mechanic| {
                let binding = bindings.iter().find_map(|(surface, binding)| {
                    (*surface == mechanic.semantic_surface()).then_some(*binding)
                })?;
                mechanic.into_paint_basis(binding)
            })
            .collect::<Vec<_>>()
            .into();
        self
    }

    pub(in crate::mounting) fn with_portal_overlays(
        mut self,
        portal_overlays: Vec<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
        portal_input_order: std::collections::BTreeMap<
            u64,
            crate::runtime::portal::UiPortalStackOrdinal,
        >,
    ) -> Self {
        self.portal_overlays = portal_overlays.into();
        self.portal_input_order = std::rc::Rc::new(portal_input_order);
        self
    }

    pub(in crate::mounting) fn with_portal_children(
        mut self,
        portal_children: std::collections::BTreeMap<
            worth_ui_host_contract::UiMountedInstanceIdentity,
            super::UiMountedPlacement,
        >,
    ) -> Self {
        self.portal_children = std::rc::Rc::new(portal_children);
        self
    }

    /// Where the frame presents `instance`.
    fn placement_of(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> super::UiMountedPlacement {
        self.portal_children
            .get(&instance)
            .copied()
            .unwrap_or(super::UiMountedPlacement::InPlace)
    }

    /// Admission ceiling includes commands later suppressed by Portal clipping.
    pub(in crate::mounting) fn motion_acceptance_reserved_bytes(&self) -> Option<usize> {
        let commands = self
            .semantic_text
            .len()
            .checked_add(self.portal_overlays.len())?;
        super::presentation::work_producer::motion_acceptance_reserved_bytes(commands)?
            .checked_add(self.indexed_motion_reserved_bytes?)?
            .checked_add(self.direct_scroll_reserved_bytes?)
    }

    pub(in crate::mounting) fn with_indexed_motion_reservation(
        mut self,
        bytes: Option<usize>,
    ) -> Self {
        self.indexed_motion_reserved_bytes = bytes;
        self
    }

    pub(in crate::mounting) fn with_direct_scroll_reservation(mut self, count: usize) -> Self {
        self.direct_scroll_reserved_bytes =
            crate::runtime::scroll::UiPreparedScrollDirectSuccession::reserved_bytes(count);
        self
    }

    pub(crate) fn retained_structural_bytes(&self) -> Option<usize> {
        #[cfg(test)]
        if let Some(materialized) = &self.materialized {
            return materialized.hit_test.len().checked_mul(std::mem::size_of::<
                worth_ui_host_contract::UiMountedHitTestMechanic,
            >());
        }
        self.hit_test
            .retained_structural_bytes()?
            .checked_add(self.presented_hits.retained_structural_bytes()?)?
            .checked_add(
                self.portal_input_order
                    .len()
                    .checked_mul(std::mem::size_of::<(
                        u64,
                        crate::runtime::portal::UiPortalStackOrdinal,
                    )>())?,
            )?
            .checked_add(self.semantic_text.retained_structural_bytes()?)?
            .checked_add(self.text_clips.len().checked_mul(std::mem::size_of::<(
                worth_ui_host_contract::UiMountedInstanceIdentity,
                crate::mounting::projection::UiMountedAppearanceClip,
            )>())?)?
            .checked_add(
                self.appearance_paint
                    .len()
                    .checked_mul(std::mem::size_of::<super::UiMountedAppearancePaintBasis>())?,
            )?
            .checked_add(self.portal_overlays.len().checked_mul(std::mem::size_of::<
                worth_ui_host_contract::UiMountedPortalOverlayMechanic,
            >())?)?
            .checked_add(
                self.portal_children
                    .len()
                    .checked_mul(std::mem::size_of::<(
                        worth_ui_host_contract::UiMountedInstanceIdentity,
                        super::UiMountedPlacement,
                    )>())?,
            )
    }
}

impl UiMountedUnsupportedPaintBasis {
    #[cfg(test)]
    pub(crate) const fn new(
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
        bounds: worth_ui_host_contract::UiMountedCanonicalBox,
        clip: worth_ui_host_contract::UiMountedCanonicalBox,
        semantic_order: u32,
        source_digest: u64,
    ) -> Self {
        Self {
            node_receipt,
            bounds,
            clip,
            semantic_order,
            source_digest,
        }
    }

    pub(crate) const fn node_receipt(self) -> worth_ui_host_contract::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub(crate) const fn bounds(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.bounds
    }

    pub(crate) const fn clip(self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.clip
    }

    pub(crate) const fn semantic_order(self) -> u32 {
        self.semantic_order
    }

    pub(crate) const fn source_digest(self) -> u64 {
        self.source_digest
    }
}
