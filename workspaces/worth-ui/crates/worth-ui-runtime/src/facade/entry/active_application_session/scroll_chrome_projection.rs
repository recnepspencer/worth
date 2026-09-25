//! Projecting the scroll chrome of every Scroll region occurrence on one
//! semantic surface.
//!
//! Chrome is derived, never stored: the track and thumb rectangles come from
//! the stationary viewport box, the reconciled bounds, the mounted pose offset
//! and the region's declared chrome contract. The mounted pose follows only
//! samples a witness displayed, so the thumb reports where the content already
//! is, not where the semantic target is heading.

/// The chrome one Scroll region occurrence presents this frame, bound to the
/// mounted occurrence that owns it and the appearance roles that paint it.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::facade::entry) struct UiScrollRegionChromeFacts {
    owner_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    slot: usize,
    owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
    viewport: worth_ui_host_contract::UiMountedCanonicalBox,
    pointer_clip: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
    mounted_offset: crate::runtime::scroll::UiScrollOffset,
    facts: crate::runtime::scroll::chrome::UiScrollChromeFacts,
    track_role: worth_ui_dsl::UiAppearanceRoleIdentity,
    thumb_role: worth_ui_dsl::UiAppearanceRoleIdentity,
}

impl UiScrollRegionChromeFacts {
    pub(in crate::facade::entry) fn admits_pointer(&self, point: [f32; 2]) -> bool {
        self.pointer_clip
            .is_none_or(|clip| crate::runtime::scroll::chrome::rect_contains(clip, point))
    }

    pub(in crate::facade::entry) const fn owner_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.owner_instance
    }

    /// The chain-owning occurrence the ownership chain was resolved against,
    /// which is the instance every route and pose call is keyed by.
    pub(in crate::facade::entry) const fn mounted_instance(
        &self,
    ) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.mounted_instance
    }

    /// This owner's position in that chain.
    pub(in crate::facade::entry) const fn slot(&self) -> usize {
        self.slot
    }

    pub(in crate::facade::entry) const fn owner(
        &self,
    ) -> crate::runtime::scroll::UiScrollOwnerIdentity {
        self.owner
    }

    pub(in crate::facade::entry) const fn incarnation(
        &self,
    ) -> crate::runtime::scroll::UiScrollOwnerIncarnation {
        self.incarnation
    }

    /// The region's viewport box: the box the chrome is clipped to and the box
    /// the gutter was reserved out of.
    pub(in crate::facade::entry) const fn viewport(
        &self,
    ) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.viewport
    }

    /// The offset these rectangles were derived at. Every interaction that
    /// starts from this chrome starts from this offset, so a drag and the thumb
    /// it moves never read two different frames.
    pub(in crate::facade::entry) const fn mounted_offset(
        &self,
    ) -> crate::runtime::scroll::UiScrollOffset {
        self.mounted_offset
    }

    pub(in crate::facade::entry) const fn facts(
        &self,
    ) -> &crate::runtime::scroll::chrome::UiScrollChromeFacts {
        &self.facts
    }

    pub(in crate::facade::entry) const fn track_role(
        &self,
    ) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.track_role
    }

    pub(in crate::facade::entry) const fn thumb_role(
        &self,
    ) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.thumb_role
    }
}

impl super::super::WorthUiActiveApplicationSession {
    /// The chrome every Scroll region occurrence on `surface` presents at its
    /// accepted offset. A region whose enabled axes do not overflow presents
    /// none and is absent from the result.
    pub(in crate::facade::entry) fn scroll_chrome_facts(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Vec<UiScrollRegionChromeFacts> {
        self.resolve_scroll_chrome_facts(surface, false)
    }

    pub(in crate::facade::entry) fn presented_scroll_chrome_facts(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Vec<UiScrollRegionChromeFacts> {
        self.resolve_scroll_chrome_facts(surface, true)
    }

    fn resolve_scroll_chrome_facts(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        presented: bool,
    ) -> Vec<UiScrollRegionChromeFacts> {
        let Some(scroll) = self.scroll.as_ref() else {
            return Vec::new();
        };
        scroll
            .ownership_instances()
            .flat_map(|mounted_instance| {
                self.region_chrome_on_surface(surface, mounted_instance, presented)
                    .into_iter()
            })
            .collect()
    }

    /// The chrome of each declared region occurrence in one mounted instance's
    /// ownership chain that sits on `surface`.
    fn region_chrome_on_surface(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        presented: bool,
    ) -> Vec<UiScrollRegionChromeFacts> {
        let Some(scroll) = self.scroll.as_ref() else {
            return Vec::new();
        };
        let Ok(chain) = scroll.ownership_chain(mounted_instance) else {
            return Vec::new();
        };
        chain
            .owners()
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, owner)| {
                owner.semantic_surface() == surface
                    && matches!(
                        owner,
                        crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. }
                    )
            })
            .filter_map(|(slot, owner)| {
                self.region_chrome(owner, mounted_instance, slot, presented)
            })
            .collect()
    }

    /// One region occurrence's chrome, derived at its accepted offset.
    fn region_chrome(
        &self,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: usize,
        presented: bool,
    ) -> Option<UiScrollRegionChromeFacts> {
        // An inadmissible declaration paints nothing: the denial was raised at
        // the admission site before any geometry was derived, and a region
        // whose chrome cannot be admitted presents none rather than presenting
        // chrome built from roles or an ownership the declaration never had.
        let admitted = self.admitted_scroll_chrome(owner).ok()?;
        let (owner_instance, mut content, mut viewport) = self
            .mounted
            .scroll_region_geometry(mounted_instance, slot)?;
        let scroll = self.scroll.as_ref()?;
        let retained = presented
            && (scroll.has_unpresented_layout(owner.semantic_surface())
                || scroll.has_pending_direct(owner.semantic_surface()));
        let pointer_clip = if retained {
            let target = crate::runtime::motion::UiMotionTargetIdentity::from_scroll_region_owner(
                owner.semantic_surface(),
                mounted_instance,
                super::scroll_transition_preparation::scroll_motion_owner_key(owner),
            );
            let (accepted_content, accepted_viewport, clip) =
                self.mounted.retained_scroll_chrome_geometry(target)?;
            content = accepted_content;
            viewport = accepted_viewport;
            Some(clip)
        } else {
            None
        };
        let bounds =
            crate::runtime::scroll::UiScrollBounds::from_mounted_region(content, viewport)?;
        let incarnation = self.scroll_region_incarnation(mounted_instance, slot)?;
        // The mounted pose, not the semantic offset: the thumb reports where
        // the content already is. Under the immediate policy the two coincide;
        // under a settling one the pose is the last displayed sample and the
        // semantic offset is still travelling toward it.
        let mounted_offset = if retained {
            scroll.offset(owner, incarnation).ok()?
        } else {
            self.mounted
                .mounted_scroll_pose(mounted_instance, owner_instance)
                .or_else(|| scroll.offset(owner, incarnation).ok())?
        };
        let facts = crate::runtime::scroll::chrome::UiScrollChromeFacts::derive(
            viewport,
            bounds,
            mounted_offset,
            &admitted,
        )?;
        Some(UiScrollRegionChromeFacts {
            owner_instance,
            mounted_instance,
            slot,
            incarnation,
            viewport,
            pointer_clip,
            mounted_offset,
            owner,
            facts,
            track_role: admitted.track_role().clone(),
            thumb_role: admitted.thumb_role().clone(),
        })
    }
}
