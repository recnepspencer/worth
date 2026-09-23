//! The mounted appearance mechanic that paints one scrollbar part.
//!
//! Scroll chrome is a derived paint element, not an authored node: it has no
//! graph node, no plan index and no mounted instance of its own. What it does
//! have is the mounted occurrence that reserved its gutter, so this mechanic is
//! keyed by that occurrence plus the axis and part it paints. That keying is
//! what lets one region present four rectangles in one frame without any of
//! them pretending to be a node receipt.
//!
//! Like the backdrop mechanic this is inert transport. It carries an already
//! snapped rectangle, the clip the region imposes, the resolved background and
//! corner radii, and an attribution whose digests prove which declared role and
//! which appearance state the rectangle was resolved from. It grants no
//! publication authority and never participates in hit testing: the pointer is
//! answered from the unsnapped accepted offset, not from the painted frame.

/// The axis whose overflow one chrome rectangle reports.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiMountedScrollChromeAxis {
    Inline,
    Block,
}

/// The painted part one chrome rectangle is.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiMountedScrollChromePart {
    /// The reserved gutter strip the thumb travels along, painted first.
    Track,
    /// The bar whose length reports the viewport proportion, painted on the
    /// track.
    Thumb,
}

impl UiMountedScrollChromePart {
    /// Both parts in paint order. The track is under the thumb, so a host that
    /// replays this order paints the same scrollbar the runtime lowered.
    pub const PAINT_ORDER: [Self; 2] = [Self::Track, Self::Thumb];

    /// Where this part sits within its own owner's chrome. Hosts that sort
    /// their draw list by a single integer use this rather than re-deriving the
    /// order from the part name.
    pub const fn paint_ordinal(self) -> u32 {
        match self {
            Self::Track => 0,
            Self::Thumb => 1,
        }
    }
}

/// Inert host correlation identity for one painted chrome rectangle.
///
/// The mounted instance is the Scroll region occurrence that owns the bar. It
/// is a scope anchor only: it grants the chrome no node lifecycle, no receipt
/// and no publication authority of its own.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiMountedScrollChromeIdentity {
    owner_instance: crate::UiMountedInstanceIdentity,
    axis: UiMountedScrollChromeAxis,
    part: UiMountedScrollChromePart,
}

impl UiMountedScrollChromeIdentity {
    #[doc(hidden)]
    pub const fn from_runtime_mounting(
        owner_instance: crate::UiMountedInstanceIdentity,
        axis: UiMountedScrollChromeAxis,
        part: UiMountedScrollChromePart,
    ) -> Self {
        Self {
            owner_instance,
            axis,
            part,
        }
    }

    pub const fn owner_instance(self) -> crate::UiMountedInstanceIdentity {
        self.owner_instance
    }
    pub const fn axis(self) -> UiMountedScrollChromeAxis {
        self.axis
    }
    pub const fn part(self) -> UiMountedScrollChromePart {
        self.part
    }
}

/// Inert host transport attribution for one chrome rectangle.
///
/// The role digest projects the declared appearance role the part was painted
/// from; the appearance digest projects the resolved state classes and value.
/// Neither is the runtime-owned appearance projection, and lineage here only
/// prevents rectangles resolved for different roles or different interaction
/// states from being treated as the same painted fact.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiMountedScrollChromeAppearanceAttribution {
    semantic_surface: crate::UiSemanticSurfaceIdentity,
    owner_instance: crate::UiMountedInstanceIdentity,
    role_digest: u64,
    appearance_digest: u64,
}

impl UiMountedScrollChromeAppearanceAttribution {
    #[doc(hidden)]
    pub const fn from_runtime_transport(
        semantic_surface: crate::UiSemanticSurfaceIdentity,
        owner_instance: crate::UiMountedInstanceIdentity,
        role_digest: u64,
        appearance_digest: u64,
    ) -> Option<Self> {
        if role_digest == 0 || appearance_digest == 0 {
            None
        } else {
            Some(Self {
                semantic_surface,
                owner_instance,
                role_digest,
                appearance_digest,
            })
        }
    }

    pub const fn semantic_surface(self) -> crate::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }
    pub const fn owner_instance(self) -> crate::UiMountedInstanceIdentity {
        self.owner_instance
    }
    pub const fn role_digest(self) -> u64 {
        self.role_digest
    }
    pub const fn appearance_digest(self) -> u64 {
        self.appearance_digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedScrollChromeMechanic {
    identity: UiMountedScrollChromeIdentity,
    semantic_surface: crate::UiSemanticSurfaceIdentity,
    rect: super::UiAppearanceAllocationBounds,
    clip: super::UiAppearanceClip,
    background: super::UiMountedAppearanceColor,
    radii: super::UiAppearanceNormalizedLogicalRadii,
    opacity: crate::UiMountedPresentationOpacity,
    attribution: UiMountedScrollChromeAppearanceAttribution,
}

#[doc(hidden)]
pub struct UiMountedScrollChromeCompletionInput {
    pub identity: UiMountedScrollChromeIdentity,
    pub semantic_surface: crate::UiSemanticSurfaceIdentity,
    pub rect: super::UiAppearanceAllocationBounds,
    pub clip: super::UiAppearanceClip,
    pub background: super::UiMountedAppearanceColor,
    pub radii: super::UiAppearanceNormalizedLogicalRadii,
    pub opacity: crate::UiMountedPresentationOpacity,
    pub attribution: UiMountedScrollChromeAppearanceAttribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedScrollChromeCompletionDenial {
    /// The attribution was completed for a different surface than the mechanic.
    AttributionSurfaceMismatch,
    /// The attribution was completed for a different occurrence than the one
    /// the identity names.
    AttributionOwnerMismatch,
    /// The normalized radii were computed against a different rectangle than
    /// the one being painted.
    RadiiRectangleMismatch,
    /// The clip leaves nothing of the rectangle. A chrome part the region does
    /// not show is absent, never painted at zero coverage.
    ClipExcludesRectangle,
}

impl UiMountedScrollChromeMechanic {
    #[doc(hidden)]
    pub fn complete_from_runtime_mounting(
        input: UiMountedScrollChromeCompletionInput,
    ) -> Result<Self, UiMountedScrollChromeCompletionDenial> {
        if input.attribution.semantic_surface != input.semantic_surface {
            return Err(UiMountedScrollChromeCompletionDenial::AttributionSurfaceMismatch);
        }
        if input.attribution.owner_instance != input.identity.owner_instance {
            return Err(UiMountedScrollChromeCompletionDenial::AttributionOwnerMismatch);
        }
        if !input.radii.matches_allocation(input.rect) {
            return Err(UiMountedScrollChromeCompletionDenial::RadiiRectangleMismatch);
        }
        if !clip_covers_any_of(input.clip, input.rect) {
            return Err(UiMountedScrollChromeCompletionDenial::ClipExcludesRectangle);
        }
        Ok(Self {
            identity: input.identity,
            semantic_surface: input.semantic_surface,
            rect: input.rect,
            clip: input.clip,
            background: input.background,
            radii: input.radii,
            opacity: input.opacity,
            attribution: input.attribution,
        })
    }

    pub const fn identity(&self) -> UiMountedScrollChromeIdentity {
        self.identity
    }
    pub const fn semantic_surface(&self) -> crate::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }
    /// The rectangle to paint, already snapped to the device grid by the
    /// runtime's presentation snapping.
    pub const fn rect(&self) -> super::UiAppearanceAllocationBounds {
        self.rect
    }
    pub const fn clip(&self) -> super::UiAppearanceClip {
        self.clip
    }
    pub const fn background(&self) -> super::UiMountedAppearanceColor {
        self.background
    }
    pub const fn radii(&self) -> super::UiAppearanceNormalizedLogicalRadii {
        self.radii
    }
    pub const fn opacity(&self) -> crate::UiMountedPresentationOpacity {
        self.opacity
    }
    pub const fn attribution(&self) -> UiMountedScrollChromeAppearanceAttribution {
        self.attribution
    }
    /// Hit testing answers from the unsnapped accepted offset, never from the
    /// painted rectangle, so chrome paint takes no part in it.
    pub const fn participates_in_hit_testing(&self) -> bool {
        false
    }

    /// A digest of exactly the fields that change what a host paints.
    ///
    /// Two mechanics with the same identity and the same paint digest produce
    /// the same pixels, which is what lets a host retain its prepared primitive
    /// across a frame that only moved another part.
    pub fn paint_digest(&self) -> u64 {
        let mut digest = 0x7363_726f_6c6c_5f63_u64;
        digest = fold(digest, u64::from(self.identity.part.paint_ordinal()));
        digest = fold(
            digest,
            match self.identity.axis {
                UiMountedScrollChromeAxis::Inline => 1,
                UiMountedScrollChromeAxis::Block => 2,
            },
        );
        digest = fold_rect(
            digest,
            self.rect.x(),
            self.rect.y(),
            self.rect.width(),
            self.rect.height(),
        );
        digest = fold_rect(
            digest,
            self.clip.x(),
            self.clip.y(),
            self.clip.width(),
            self.clip.height(),
        );
        digest = self
            .background
            .straight_srgba()
            .iter()
            .fold(digest, |digest, channel| fold(digest, u64::from(*channel)));
        digest = self
            .radii
            .corners()
            .iter()
            .fold(digest, |digest, corner| fold(digest, u64::from(*corner)));
        fold(digest, u64::from(self.opacity.units()))
    }
}

const fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

const fn fold_rect(digest: u64, x: i32, y: i32, width: u32, height: u32) -> u64 {
    let digest = fold(digest, x as u32 as u64);
    let digest = fold(digest, y as u32 as u64);
    let digest = fold(digest, width as u64);
    fold(digest, height as u64)
}

const fn clip_covers_any_of(
    clip: super::UiAppearanceClip,
    rect: super::UiAppearanceAllocationBounds,
) -> bool {
    let clip_right = clip.x() as i64 + clip.width() as i64;
    let clip_bottom = clip.y() as i64 + clip.height() as i64;
    let rect_right = rect.x() as i64 + rect.width() as i64;
    let rect_bottom = rect.y() as i64 + rect.height() as i64;
    let left = if clip.x() as i64 > rect.x() as i64 {
        clip.x() as i64
    } else {
        rect.x() as i64
    };
    let top = if clip.y() as i64 > rect.y() as i64 {
        clip.y() as i64
    } else {
        rect.y() as i64
    };
    let right = if clip_right < rect_right {
        clip_right
    } else {
        rect_right
    };
    let bottom = if clip_bottom < rect_bottom {
        clip_bottom
    } else {
        rect_bottom
    };
    left < right && top < bottom
}

#[cfg(test)]
#[path = "scroll_chrome_tests.rs"]
mod tests;
