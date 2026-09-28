//! Bounded runtime-owned evidence for one mounted text presentation turn.

use super::rasterization::UiNativeTextRasterWorkReport;
use super::UiNativeTextPresentationPrepared;
use sha2::{Digest, Sha256};

#[path = "work_observation/transcript.rs"]
mod transcript;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeTextPresentationMechanicObservation {
    mechanic: worth_ui_host_contract::UiMountedPaintCommandIdentity,
    layout_digest: [u8; 32],
    raster_key_set_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiNativeTextPresentationWorkObservation {
    attempt: u64,
    binding: u64,
    mounted_frame: u64,
    host_lineage: u64,
    dpi_milli: u32,
    layout_count: u32,
    paint_span_count: u32,
    demand_batches: u32,
    demand_records: u32,
    key_checks: u32,
    rasterized_glyphs: u32,
    rasterized_texels: u64,
    produced_bytes: u64,
    pin_additions: u32,
    pin_releases: u32,
    binding_pins: u32,
    removed_mechanic_count: u32,
    performed_layout_work: [u64; 17],
    transcript: transcript::UiNativeTextTranscriptEvidence,
    mechanic_identity_digests: [[u8; 32]; 2],
    binding_pin_identities: Box<[[[u8; 32]; 2]]>,
    active_mechanics: Box<[UiNativeTextPresentationMechanicObservation]>,
    removed_mechanics: Box<[UiNativeTextPresentationMechanicObservation]>,
}

impl UiNativeTextPresentationWorkObservation {
    pub(super) fn after_mounted_work(
        basis: &worth_ui_query_binding::WorthUiPresentationRequestBasis,
        prepared: &UiNativeTextPresentationPrepared,
        raster: UiNativeTextRasterWorkReport,
        performed_layout_work: [u64; 17],
        active_mechanics: Box<[UiNativeTextPresentationMechanicObservation]>,
        removed_mechanics: Box<[UiNativeTextPresentationMechanicObservation]>,
    ) -> Self {
        let planning = prepared
            .planning_inspection()
            .expect("prepared native text retains its demand inspection");
        let transcript = transcript::UiNativeTextTranscriptEvidence::from_prepared(prepared);
        Self {
            attempt: basis.attempt().diagnostic_value(),
            binding: basis.binding().diagnostic_value(),
            mounted_frame: basis.mounted_frame().diagnostic_value(),
            host_lineage: basis.host_lineage().diagnostic_value(),
            dpi_milli: basis.dpi_milli(),
            layout_count: bounded_u32(prepared.layout_count()),
            paint_span_count: bounded_u32(prepared.paint_span_count()),
            demand_batches: planning.demand_batches(),
            demand_records: planning.demand_records(),
            key_checks: planning.key_checks(),
            rasterized_glyphs: raster.rasterized_glyphs(),
            rasterized_texels: raster.rasterized_texels(),
            produced_bytes: raster.produced_bytes(),
            pin_additions: bounded_u32(basis.pin_additions().len()),
            pin_releases: bounded_u32(basis.pin_releases().len()),
            binding_pins: bounded_u32(basis.binding_pins().len()),
            removed_mechanic_count: bounded_u32(basis.removed_mechanics().len()),
            performed_layout_work,
            transcript,
            mechanic_identity_digests: [
                basis.active_mechanic_identity_digest(),
                basis.removed_mechanic_identity_digest(),
            ],
            binding_pin_identities: basis
                .binding_pins()
                .iter()
                .map(|pin| {
                    [
                        pin.layout().digest(),
                        Sha256::digest(pin.key().canonical_evidence_bytes()).into(),
                    ]
                })
                .collect(),
            active_mechanics,
            removed_mechanics,
        }
    }

    pub(crate) const fn identity(&self) -> [u64; 4] {
        [
            self.attempt,
            self.binding,
            self.mounted_frame,
            self.host_lineage,
        ]
    }

    pub(crate) fn work_counts(&self) -> [u64; 30] {
        let mut counts = [0; 30];
        counts[..13].copy_from_slice(&[
            self.dpi_milli as u64,
            self.layout_count as u64,
            self.paint_span_count as u64,
            self.demand_batches as u64,
            self.demand_records as u64,
            self.key_checks as u64,
            self.rasterized_glyphs as u64,
            self.rasterized_texels,
            self.produced_bytes,
            self.pin_additions as u64,
            self.pin_releases as u64,
            self.binding_pins as u64,
            self.removed_mechanic_count as u64,
        ]);
        counts[13..].copy_from_slice(&self.performed_layout_work);
        counts
    }

    pub(crate) const fn transcript_digests(&self) -> [[u8; 32]; 4] {
        self.transcript.digests()
    }

    pub(crate) const fn intrinsic_glyph_runs(&self) -> u64 {
        self.transcript.intrinsic_glyph_runs()
    }

    pub(crate) const fn mechanic_identity_digests(&self) -> [[u8; 32]; 2] {
        self.mechanic_identity_digests
    }

    pub(crate) fn binding_pin_identities(&self) -> &[[[u8; 32]; 2]] {
        &self.binding_pin_identities
    }

    pub(crate) fn active_mechanics(&self) -> &[UiNativeTextPresentationMechanicObservation] {
        &self.active_mechanics
    }

    pub(crate) fn removed_mechanics(&self) -> &[UiNativeTextPresentationMechanicObservation] {
        &self.removed_mechanics
    }
}

impl UiNativeTextPresentationMechanicObservation {
    pub(crate) fn from_basis(
        mechanic: &worth_ui_query_binding::WorthUiPresentationMechanicBasis,
    ) -> Self {
        Self {
            mechanic: mechanic.mechanic(),
            layout_digest: mechanic.layout().digest(),
            raster_key_set_digest: raster_key_set_digest(mechanic.raster_key_set().keys()),
        }
    }

    pub(crate) const fn mechanic(self) -> worth_ui_host_contract::UiMountedPaintCommandIdentity {
        self.mechanic
    }

    pub(crate) const fn layout_digest(self) -> [u8; 32] {
        self.layout_digest
    }

    pub(crate) const fn raster_key_set_digest(self) -> [u8; 32] {
        self.raster_key_set_digest
    }
}

fn raster_key_set_digest(keys: &[worth_ui_host_contract::UiGlyphRasterKey]) -> [u8; 32] {
    let mut rows = keys
        .iter()
        .map(|key| key.canonical_evidence_bytes())
        .collect::<Vec<_>>();
    rows.sort_unstable();
    let mut digest = Sha256::new();
    digest.update((rows.len() as u64).to_le_bytes());
    for row in rows {
        digest.update((row.len() as u64).to_le_bytes());
        digest.update(row);
    }
    digest.finalize().into()
}

fn bounded_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
