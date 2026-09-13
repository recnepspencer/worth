#[derive(Clone)]
pub(crate) struct UiMountedVisualRegionBasis {
    hit_test: crate::mounting::projection::UiMountedHitMechanicSource,
    pub(in crate::mounting) presented_hits:
        crate::mounting::presented_hit_index::UiPresentedHitIndex,
    semantic_text: crate::mounting::projection::UiMountedSemanticMechanicSource,
    portal_overlays: std::rc::Rc<[worth_ui_host_contract::UiMountedPortalOverlayMechanic]>,
    portal_children: std::rc::Rc<
        std::collections::BTreeMap<
            worth_ui_host_contract::UiMountedInstanceIdentity,
            Option<(
                worth_ui_host_contract::UiMountedPortalOverlayMechanic,
                worth_ui_host_contract::UiMountedCanonicalBox,
            )>,
        >,
    >,
    appearance_paint: std::sync::Arc<[super::UiMountedAppearancePaintBasis]>,
    binding: Option<worth_ui_host_contract::UiSurfaceBindingGeneration>,
    receipts: Option<super::UiMountedNodeReceiptBasis>,
    #[cfg(test)]
    materialized: Option<UiMaterializedVisualRegionBasis>,
}

pub(crate) struct UiMountedHitTestPresentation {
    mechanic: worth_ui_host_contract::UiMountedHitTestMechanic,
    portal: Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
    owns_presented_portal: bool,
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
            portal_children: Default::default(),
            appearance_paint: std::sync::Arc::from([]),
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
            portal_children: Default::default(),
            appearance_paint: std::sync::Arc::from([]),
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
            portal_children: std::rc::Rc::clone(&self.portal_children),
            appearance_paint: std::sync::Arc::clone(&self.appearance_paint),
            binding: Some(binding),
            receipts: Some(receipts),
            #[cfg(test)]
            materialized: self.materialized.clone(),
        }
    }

    pub(crate) fn hit_test(&self) -> Box<[UiMountedHitTestPresentation]> {
        #[cfg(test)]
        if let Some(materialized) = &self.materialized {
            return materialized
                .hit_test
                .iter()
                .copied()
                .map(|mechanic| UiMountedHitTestPresentation {
                    mechanic,
                    portal: None,
                    owns_presented_portal: false,
                })
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
                let (mechanic, portal) = match self.portal_children.get(&row.mounted_instance()) {
                    None => Some((row, None)),
                    Some(None) => None,
                    Some(Some((portal, source_anchor))) => Some((
                        row.presented_within_portal(*portal, *source_anchor)
                            .expect("validated Portal-relative hit region remains canonical")?,
                        Some(*portal),
                    )),
                }?;
                Some(UiMountedHitTestPresentation {
                    mechanic,
                    portal,
                    owns_presented_portal: portal_owners.contains(&mechanic.mounted_instance()),
                })
            })
            .collect()
    }

    pub(crate) fn unsupported_paint(&self) -> Box<[UiMountedUnsupportedPaintBasis]> {
        #[cfg(test)]
        if let Some(materialized) = &self.materialized {
            return materialized.unsupported_paint.iter().copied().collect();
        }
        self.semantic_text
            .visual_mechanics()
            .filter(|row| self.binding.is_none_or(|binding| row.binding() == binding))
            .filter_map(|row| {
                let presented = match self.portal_children.get(&row.mounted_instance()) {
                    None => Some(row.clone()),
                    Some(None) => None,
                    Some(Some((portal, source_anchor))) => row
                        .presented_within_portal(*portal, *source_anchor)
                        .expect("validated Portal-relative text remains canonical"),
                }?;
                Some(UiMountedUnsupportedPaintBasis {
                    node_receipt: self
                        .receipts
                        .as_ref()
                        .and_then(|receipts| receipts.receipt_for(presented.mounted_instance()))
                        .unwrap_or_else(|| presented.node_receipt()),
                    bounds: presented.bounds(),
                    clip: presented.clip_bounds(),
                    semantic_order: presented.layer_semantic_order(),
                    source_digest: presented.semantic_digest(),
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
    ) -> Self {
        self.portal_overlays = portal_overlays.into();
        self
    }

    pub(in crate::mounting) fn with_portal_children(
        mut self,
        portal_children: std::collections::BTreeMap<
            worth_ui_host_contract::UiMountedInstanceIdentity,
            Option<(
                worth_ui_host_contract::UiMountedPortalOverlayMechanic,
                worth_ui_host_contract::UiMountedCanonicalBox,
            )>,
        >,
    ) -> Self {
        self.portal_children = std::rc::Rc::new(portal_children);
        self
    }

    /// Admission ceiling includes commands later suppressed by Portal clipping.
    pub(in crate::mounting) fn motion_acceptance_reserved_bytes(&self) -> Option<usize> {
        let commands = self
            .semantic_text
            .len()
            .checked_add(self.portal_overlays.len())?;
        super::presentation::work_producer::motion_acceptance_reserved_bytes(commands)
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
            .checked_add(self.semantic_text.retained_structural_bytes()?)?
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
                        Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
                    )>())?,
            )
    }
}

impl UiMountedHitTestPresentation {
    pub(in crate::mounting) fn completed(
        mechanic: worth_ui_host_contract::UiMountedHitTestMechanic,
        portal: Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic>,
        owns_presented_portal: bool,
    ) -> Self {
        Self {
            mechanic,
            portal,
            owns_presented_portal,
        }
    }
    pub(crate) const fn mechanic(&self) -> worth_ui_host_contract::UiMountedHitTestMechanic {
        self.mechanic
    }

    pub(crate) const fn portal(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedPortalOverlayMechanic> {
        self.portal
    }

    pub(crate) const fn owns_presented_portal(&self) -> bool {
        self.owns_presented_portal
    }

    #[cfg(test)]
    pub(crate) const fn for_test(
        mechanic: worth_ui_host_contract::UiMountedHitTestMechanic,
    ) -> Self {
        Self {
            mechanic,
            portal: None,
            owns_presented_portal: false,
        }
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
