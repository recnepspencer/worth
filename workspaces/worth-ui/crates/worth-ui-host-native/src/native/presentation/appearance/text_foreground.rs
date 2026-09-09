//! Staged text foreground finalization from authenticated demand and atlas images.

mod paint;

use super::damage::{UiNativeAppearanceDamage, UiNativeAppearanceDamageRect};
use crate::native::presentation::text::{
    mechanic_contains_run, plan_glyph_commands, UiNativeGlyphCommandDenial,
};
use crate::native::text_atlas::{UiNativeTextAtlas, UiNativeTextAtlasImageObservation};
use worth_ui_host_contract::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeTextForegroundFinalizationDenial {
    FragmentAffinity,
    MissingForeground,
    MissingCandidate,
    CandidateLayout,
    CandidateAttribution,
    MissingRasterWork,
    UnauthenticatedDemand,
    IntrinsicColor,
    MissingAtlasEntry,
    AtlasImageBasis,
    PresentationCoverage,
    Geometry,
    CoverageCapacity,
    AtlasAdmission,
    RasterAdmission,
}

/// Successful staged join observations, not a total-work or locality claim.
/// Attribution's nested glyph/span scans and failed demand comparisons are not
/// included; accepted-path integration must account for those costs separately.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiNativeTextForegroundJoinCost {
    pub candidate_visits: usize,
    pub run_visits: usize,
    pub demand_validation_attempts: usize,
    pub authenticated_records: usize,
    pub image_commands: usize,
}

/// Private construction prevents semantic-only foreground from entering retention.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiNativeFinalizedTextForeground {
    mechanic: UiMountedTextForegroundAppearanceMechanic,
    binding: UiMountedSurfaceBindingRequirement,
    affinity: UiMountedPresentationAffinity,
    attempt: UiMountedPresentationAttemptIdentity,
    coverage: Box<[UiNativeAppearanceDamageRect]>,
    images: UiNativeTextAtlasImageObservation,
    candidates: Box<[UiMountedSemanticTextMechanic]>,
    glyphs: Box<[crate::native::presentation::text::UiNativeGlyphCommand]>,
}

pub(crate) struct UiNativeTextForegroundJoin {
    mechanic: UiMountedTextForegroundAppearanceMechanic,
    binding: UiMountedSurfaceBindingRequirement,
    affinity: UiMountedPresentationAffinity,
    attempt: UiMountedPresentationAttemptIdentity,
    runs: Vec<UiGlyphRunView>,
    candidates: Vec<UiMountedSemanticTextMechanic>,
    cost: UiNativeTextForegroundJoinCost,
}

impl UiNativeTextForegroundJoin {
    pub(crate) fn admit(
        fragment: &UiUnpublishedAppearanceFragment,
        view: &UiMountedFrameConsumptionView<'_>,
        mechanic: &UiMountedTextForegroundAppearanceMechanic,
    ) -> Result<Self, UiNativeTextForegroundFinalizationDenial> {
        use UiNativeTextForegroundFinalizationDenial as Denial;
        if fragment.surface_binding() != view.requirement()
            || fragment.presentation_affinity() != view.presentation_work().affinity()
            || fragment.work().successor().overlay_order().presentation() != view.attempt()
            || fragment.surface_binding().capability_generation() != view.capability_generation()
            || fragment.surface_binding().capability_profile_digest()
                != view.capability_profile_digest()
        {
            return Err(Denial::FragmentAffinity);
        }
        if !fragment.work().successor().mechanics().iter().any(|row|
            matches!(row, UiMountedAppearanceMechanic::TextForeground(value) if value == mechanic)) {
            return Err(Denial::MissingForeground);
        }
        let raster = view.text_raster_work().ok_or(Denial::MissingRasterWork)?;
        let mut join = Self {
            mechanic: mechanic.clone(),
            binding: fragment.surface_binding(),
            affinity: fragment.presentation_affinity(),
            attempt: view.attempt(),
            runs: Vec::new(),
            candidates: Vec::new(),
            cost: Default::default(),
        };
        let mut matched = false;
        for candidate in fragment.text_candidates() {
            join.cost.candidate_visits += 1;
            if !consumes_foreground(candidate, mechanic) {
                continue;
            }
            matched = true;
            join.candidates.push(candidate.clone());
            let layout = view
                .qualified_text_layout(candidate)
                .ok_or(Denial::CandidateLayout)?;
            if layout.width_basis() != candidate.qualified_layout_width() {
                return Err(Denial::CandidateLayout);
            }
            let command = UiMountedPaintCommandIdentity::semantic_text(candidate);
            let runs = raster
                .glyph_runs()
                .iter()
                .copied()
                .filter(|run| run.mechanic() == command)
                .collect::<Vec<_>>();
            join.cost.run_visits += raster.glyph_runs().len();
            if runs
                .iter()
                .any(|run| !mechanic_contains_run(candidate, layout, *run))
            {
                return Err(Denial::CandidateAttribution);
            }
            let mut authenticated = false;
            for demand in raster
                .demands()
                .iter()
                .copied()
                .filter(|demand| demand.layout_identity() == layout.identity())
            {
                join.cost.demand_validation_attempts += 1;
                if demand.dpi_milli() != view.requirement().device_scale_milli() {
                    continue;
                }
                if let Ok(cost) = raster.validate_complete_demand(command, demand, &runs) {
                    join.cost.authenticated_records += cost.demand_records_checked;
                    authenticated = true;
                    break;
                }
            }
            if !authenticated {
                return Err(Denial::UnauthenticatedDemand);
            }
            for run in runs
                .into_iter()
                .filter(|run| run.paint_span() == mechanic.paint_span())
            {
                if matches!(
                    run.raster_key().source(),
                    UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap
                ) {
                    return Err(Denial::IntrinsicColor);
                }
                join.runs.push(run);
            }
        }
        if !matched {
            return Err(Denial::MissingCandidate);
        }
        Ok(join)
    }

    pub(crate) fn finalize(
        mut self,
        atlas: &UiNativeTextAtlas,
        extent: [u32; 2],
    ) -> Result<
        (
            UiNativeFinalizedTextForeground,
            UiNativeTextForegroundJoinCost,
        ),
        UiNativeTextForegroundFinalizationDenial,
    > {
        use UiNativeTextForegroundFinalizationDenial as Denial;
        let images = atlas
            .observe_images()
            .map_err(|_| Denial::AtlasImageBasis)?;
        let mut commands =
            plan_glyph_commands(&self.runs, atlas, extent).map_err(|denial| match denial {
                UiNativeGlyphCommandDenial::MissingAtlasEntry => Denial::MissingAtlasEntry,
                UiNativeGlyphCommandDenial::GeometryOverflow => Denial::Geometry,
            })?;
        let mut coverage = UiNativeAppearanceDamage::new(usize::from(
            crate::native_profile::STAGED_APPEARANCE_PROFILE.damage_regions,
        ));
        let [red, green, blue, alpha] = self.mechanic.foreground().straight_srgba();
        let foreground = UiMountedRgba8::new(red, green, blue, alpha);
        // Runtime already composed appearance and Motion at u16 precision.
        let opacity = f32::from(self.mechanic.opacity().units()) / f32::from(u16::MAX);
        for command in commands.iter_mut() {
            command.foreground = foreground;
            command.opacity = opacity;
            let [x, y, width, height] = command.target;
            let edges = [
                x.floor(),
                y.floor(),
                (x + width).ceil(),
                (y + height).ceil(),
            ];
            if edges
                .iter()
                .any(|edge| !edge.is_finite() || *edge < 0.0 || f64::from(*edge) > i64::MAX as f64)
            {
                return Err(Denial::Geometry);
            }
            coverage
                .add(UiNativeAppearanceDamageRect {
                    left: edges[0] as i64,
                    top: edges[1] as i64,
                    right: edges[2] as i64,
                    bottom: edges[3] as i64,
                })
                .map_err(|_| Denial::CoverageCapacity)?;
        }
        self.cost.image_commands = commands.len();
        Ok((
            UiNativeFinalizedTextForeground {
                mechanic: self.mechanic,
                binding: self.binding,
                affinity: self.affinity,
                attempt: self.attempt,
                coverage: coverage.take(),
                images,
                candidates: self.candidates.into_boxed_slice(),
                glyphs: commands,
            },
            self.cost,
        ))
    }
}

impl UiNativeFinalizedTextForeground {
    #[cfg(feature = "certification-support")]
    pub(crate) fn glyph_vertex_colors(&self, extent: [u32; 2]) -> Box<[[f32; 4]]> {
        self.glyphs()
            .iter()
            .map(|glyph| crate::native::presentation::text::glyph_vertices(*glyph, extent)[0].color)
            .collect()
    }
    pub(crate) fn glyphs(&self) -> &[crate::native::presentation::text::UiNativeGlyphCommand] {
        &self.glyphs
    }
    pub(crate) fn matches_candidates(&self, fragment: &UiUnpublishedAppearanceFragment) -> bool {
        fragment
            .text_candidates()
            .iter()
            .filter(|candidate| consumes_foreground(candidate, &self.mechanic))
            .eq(self.candidates.iter())
    }
    pub(crate) fn presentation_attempt(&self) -> UiMountedPresentationAttemptIdentity {
        self.attempt
    }
    pub(crate) fn binding(&self) -> UiMountedSurfaceBindingRequirement {
        self.binding
    }
    pub(crate) fn affinity(&self) -> UiMountedPresentationAffinity {
        self.affinity
    }
    /// Candidate image validity is separate from historical damage coverage.
    pub(crate) fn validate_images(
        &self,
        atlas: &UiNativeTextAtlas,
    ) -> Result<(), UiNativeTextForegroundFinalizationDenial> {
        atlas
            .validate_images(self.images)
            .map_err(|_| UiNativeTextForegroundFinalizationDenial::AtlasImageBasis)
    }
    pub(crate) fn mechanic(&self) -> &UiMountedTextForegroundAppearanceMechanic {
        &self.mechanic
    }
    pub(crate) fn coverage(&self) -> &[UiNativeAppearanceDamageRect] {
        &self.coverage
    }
    pub(crate) fn damage_bounds(
        &self,
        scale: super::geometry::UiNativeAppearanceScale,
    ) -> Result<Option<UiNativeAppearanceDamageRect>, super::geometry::UiNativeGeometryDenial> {
        if u32::from(scale.milli()) != self.binding.device_scale_milli() {
            return Err(super::geometry::UiNativeGeometryDenial::UnsupportedScale);
        }
        Ok(self
            .coverage
            .iter()
            .copied()
            .reduce(UiNativeAppearanceDamageRect::union))
    }
}

fn consumes_foreground(
    candidate: &UiMountedSemanticTextMechanic,
    mechanic: &UiMountedTextForegroundAppearanceMechanic,
) -> bool {
    UiMountedPaintCommandIdentity::semantic_text(candidate) == mechanic.command()
        && candidate.node_receipt() == mechanic.node_receipt()
        && candidate
            .foregrounds()
            .iter()
            .any(|span| span.identity() == mechanic.paint_span())
}
