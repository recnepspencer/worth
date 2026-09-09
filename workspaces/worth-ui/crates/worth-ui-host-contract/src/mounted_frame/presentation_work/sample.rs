use super::{UiMountedLogicalDamage, UiMountedPaintCommandIdentity, UiMountedPresentationAffinity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedPresentationSampleConstructionDenial {
    CoordinateSpaceMismatch,
    EmptyTransformSource,
    EmptyChanges,
    DuplicateCommandIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedPresentationOpacity(u16);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedPresentationTransform {
    source: crate::UiMountedCanonicalBox,
    sampled: crate::UiMountedCanonicalBox,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMountedPresentationSampleChange {
    command: UiMountedPaintCommandIdentity,
    transform: Option<UiMountedPresentationTransform>,
    opacity: UiMountedPresentationOpacity,
}

#[derive(Debug, PartialEq)]
pub struct UiMountedPresentationSample {
    affinity: UiMountedPresentationAffinity,
    changes: Box<[UiMountedPresentationSampleChange]>,
    damage: Box<[UiMountedLogicalDamage]>,
    production_cost: crate::UiMountedPresentationProductionCost,
}

#[doc(hidden)]
pub struct UiMountedPresentationSampleInput {
    pub frame: crate::UiMountedFrameIdentity,
    pub surface: crate::UiSemanticSurfaceIdentity,
    pub binding: crate::UiSurfaceBindingGeneration,
    pub content: crate::UiMountedContentGeneration,
    pub baseline: crate::UiHostSurfaceBaselineIdentity,
    pub changes: Vec<UiMountedPresentationSampleChange>,
    pub damage: Vec<UiMountedLogicalDamage>,
    pub production_cost: crate::UiMountedPresentationProductionCost,
}

impl UiMountedPresentationOpacity {
    #[doc(hidden)]
    pub const fn from_runtime_composition(units: u16) -> Self {
        Self(units)
    }

    pub const fn units(self) -> u16 {
        self.0
    }

    pub fn factor(self) -> f32 {
        f32::from(self.0) / f32::from(u16::MAX)
    }
}

impl UiMountedPresentationTransform {
    #[doc(hidden)]
    pub fn from_runtime_sampling(
        source: crate::UiMountedCanonicalBox,
        sampled: crate::UiMountedCanonicalBox,
    ) -> Result<Self, UiMountedPresentationSampleConstructionDenial> {
        if source.coordinate_space() != sampled.coordinate_space() {
            return Err(UiMountedPresentationSampleConstructionDenial::CoordinateSpaceMismatch);
        }
        if source.width() == 0.0 || source.height() == 0.0 {
            return Err(UiMountedPresentationSampleConstructionDenial::EmptyTransformSource);
        }
        Ok(Self { source, sampled })
    }

    pub const fn source(self) -> crate::UiMountedCanonicalBox {
        self.source
    }

    pub const fn sampled(self) -> crate::UiMountedCanonicalBox {
        self.sampled
    }
}

impl UiMountedPresentationSampleChange {
    #[doc(hidden)]
    pub const fn from_runtime_sampling(
        command: UiMountedPaintCommandIdentity,
        transform: Option<UiMountedPresentationTransform>,
        opacity: UiMountedPresentationOpacity,
    ) -> Self {
        Self {
            command,
            transform,
            opacity,
        }
    }

    pub const fn command(self) -> UiMountedPaintCommandIdentity {
        self.command
    }

    pub const fn transform(self) -> Option<UiMountedPresentationTransform> {
        self.transform
    }

    pub const fn opacity(self) -> UiMountedPresentationOpacity {
        self.opacity
    }
}

impl UiMountedPresentationSample {
    #[doc(hidden)]
    pub fn from_inert_mechanics(
        input: UiMountedPresentationSampleInput,
    ) -> Result<Self, UiMountedPresentationSampleConstructionDenial> {
        if input.changes.is_empty() {
            return Err(UiMountedPresentationSampleConstructionDenial::EmptyChanges);
        }
        let unique = input
            .changes
            .iter()
            .map(|change| change.command())
            .collect::<std::collections::HashSet<_>>();
        if unique.len() != input.changes.len() {
            return Err(UiMountedPresentationSampleConstructionDenial::DuplicateCommandIdentity);
        }
        let affinity = UiMountedPresentationAffinity::from_runtime(
            super::affinity::UiMountedPresentationAffinityInput {
                predecessor: Some(input.frame),
                successor: input.frame,
                surface: input.surface,
                binding: input.binding,
                content: input.content,
                baseline: input.baseline,
                receipt_affinity: None,
            },
        );
        Ok(Self {
            affinity,
            changes: input.changes.into_boxed_slice(),
            damage: input.damage.into_boxed_slice(),
            production_cost: input.production_cost,
        })
    }

    pub const fn affinity(&self) -> UiMountedPresentationAffinity {
        self.affinity
    }

    pub fn changes(&self) -> &[UiMountedPresentationSampleChange] {
        &self.changes
    }

    pub fn damage(&self) -> &[UiMountedLogicalDamage] {
        &self.damage
    }

    pub const fn production_cost(&self) -> crate::UiMountedPresentationProductionCost {
        self.production_cost
    }
}
