//! Owner-issued performed text work retained at client shutdown.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiNativeClientPresentationMechanicIdentityObservation {
    mounted_instance: u64,
    semantic_slot: u16,
    collection_row: Option<[u8; 32]>,
    layout_digest: [u8; 32],
    raster_key_set_digest: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativeClientTextPresentationWorkObservation {
    identity: [u64; 4],
    work_counts: [u64; 30],
    transcript_digests: [[u8; 32]; 4],
    intrinsic_glyph_runs: u64,
    mechanic_identity_digests: [[u8; 32]; 2],
    binding_pin_identities: Box<[[[u8; 32]; 2]]>,
    active_mechanics: Box<[UiNativeClientPresentationMechanicIdentityObservation]>,
    removed_mechanics: Box<[UiNativeClientPresentationMechanicIdentityObservation]>,
}

impl UiNativeClientPresentationMechanicIdentityObservation {
    pub fn reported(
        mounted_instance: u64,
        semantic_slot: u16,
        collection_row: Option<[u8; 32]>,
        layout_digest: [u8; 32],
        raster_key_set_digest: [u8; 32],
    ) -> Self {
        Self {
            mounted_instance,
            semantic_slot,
            collection_row,
            layout_digest,
            raster_key_set_digest,
        }
    }

    pub const fn mounted_instance(self) -> u64 {
        self.mounted_instance
    }

    pub const fn semantic_slot(self) -> u16 {
        self.semantic_slot
    }

    pub const fn collection_row(self) -> Option<[u8; 32]> {
        self.collection_row
    }

    pub const fn layout_digest(self) -> [u8; 32] {
        self.layout_digest
    }

    pub const fn raster_key_set_digest(self) -> [u8; 32] {
        self.raster_key_set_digest
    }
}

impl UiNativeClientTextPresentationWorkObservation {
    pub fn reported(
        identity: [u64; 4],
        work_counts: [u64; 30],
        transcript_digests: [[u8; 32]; 4],
        intrinsic_glyph_runs: u64,
        mechanic_identity_digests: [[u8; 32]; 2],
        binding_pin_identities: impl IntoIterator<Item = [[u8; 32]; 2]>,
        active_mechanics: impl IntoIterator<
            Item = UiNativeClientPresentationMechanicIdentityObservation,
        >,
        removed_mechanics: impl IntoIterator<
            Item = UiNativeClientPresentationMechanicIdentityObservation,
        >,
    ) -> Self {
        Self {
            identity,
            work_counts,
            transcript_digests,
            intrinsic_glyph_runs,
            mechanic_identity_digests,
            binding_pin_identities: binding_pin_identities.into_iter().collect(),
            active_mechanics: active_mechanics.into_iter().collect(),
            removed_mechanics: removed_mechanics.into_iter().collect(),
        }
    }

    pub const fn identity(&self) -> [u64; 4] {
        self.identity
    }

    pub const fn work_counts(&self) -> [u64; 30] {
        self.work_counts
    }

    pub const fn layout_set_digest(&self) -> [u8; 32] {
        self.transcript_digests[0]
    }

    pub const fn raster_key_set_digest(&self) -> [u8; 32] {
        self.transcript_digests[1]
    }

    pub const fn glyph_run_transcript_digest(&self) -> [u8; 32] {
        self.transcript_digests[2]
    }

    pub const fn intrinsic_glyph_transcript_digest(&self) -> [u8; 32] {
        self.transcript_digests[3]
    }

    pub const fn intrinsic_glyph_runs(&self) -> u64 {
        self.intrinsic_glyph_runs
    }

    pub const fn active_mechanic_identity_digest(&self) -> [u8; 32] {
        self.mechanic_identity_digests[0]
    }

    pub const fn removed_mechanic_identity_digest(&self) -> [u8; 32] {
        self.mechanic_identity_digests[1]
    }

    pub fn binding_pin_identities(&self) -> &[[[u8; 32]; 2]] {
        &self.binding_pin_identities
    }

    pub fn active_mechanic_identities(
        &self,
    ) -> &[UiNativeClientPresentationMechanicIdentityObservation] {
        &self.active_mechanics
    }

    pub fn removed_mechanic_identities(
        &self,
    ) -> &[UiNativeClientPresentationMechanicIdentityObservation] {
        &self.removed_mechanics
    }

    pub const fn attempt(&self) -> u64 {
        self.identity[0]
    }

    pub const fn binding(&self) -> u64 {
        self.identity[1]
    }

    pub const fn mounted_frame(&self) -> u64 {
        self.identity[2]
    }

    pub const fn host_lineage(&self) -> u64 {
        self.identity[3]
    }

    /// Whether the turn qualified, rasterized, uploaded, pinned, or released
    /// anything. The turn's DPI and the pins its binding still holds are
    /// state, not work.
    pub fn performed_work(&self) -> bool {
        self.work_counts
            .iter()
            .enumerate()
            .any(|(index, count)| index != 0 && index != 11 && *count != 0)
    }

    pub const fn dpi_milli(&self) -> u64 {
        self.work_counts[0]
    }

    pub const fn layout_count(&self) -> u64 {
        self.work_counts[1]
    }

    pub const fn paint_span_count(&self) -> u64 {
        self.work_counts[2]
    }

    pub const fn demand_batches(&self) -> u64 {
        self.work_counts[3]
    }

    pub const fn demand_records(&self) -> u64 {
        self.work_counts[4]
    }

    pub const fn key_checks(&self) -> u64 {
        self.work_counts[5]
    }

    pub const fn rasterized_glyphs(&self) -> u64 {
        self.work_counts[6]
    }

    pub const fn rasterized_texels(&self) -> u64 {
        self.work_counts[7]
    }

    pub const fn produced_bytes(&self) -> u64 {
        self.work_counts[8]
    }

    pub const fn pin_additions(&self) -> u64 {
        self.work_counts[9]
    }

    pub const fn pin_releases(&self) -> u64 {
        self.work_counts[10]
    }

    pub const fn binding_pins(&self) -> u64 {
        self.work_counts[11]
    }

    pub const fn removed_mechanics(&self) -> u64 {
        self.work_counts[12]
    }

    pub const fn analyzed_bytes(&self) -> u64 {
        self.work_counts[13]
    }

    pub const fn graphemes(&self) -> u64 {
        self.work_counts[14]
    }

    pub const fn word_boundaries(&self) -> u64 {
        self.work_counts[15]
    }

    pub const fn line_opportunities(&self) -> u64 {
        self.work_counts[16]
    }

    pub const fn bidi_contexts(&self) -> u64 {
        self.work_counts[17]
    }

    pub const fn fallback_clusters(&self) -> u64 {
        self.work_counts[18]
    }

    pub const fn coverage_index_queries(&self) -> u64 {
        self.work_counts[19]
    }

    pub const fn face_shape_attempts(&self) -> u64 {
        self.work_counts[20]
    }

    pub const fn probed_glyphs(&self) -> u64 {
        self.work_counts[21]
    }

    pub const fn shaped_runs(&self) -> u64 {
        self.work_counts[22]
    }

    pub const fn shaped_scalars(&self) -> u64 {
        self.work_counts[23]
    }

    pub const fn emitted_glyphs(&self) -> u64 {
        self.work_counts[24]
    }

    pub const fn fitted_units(&self) -> u64 {
        self.work_counts[25]
    }

    pub const fn emitted_lines(&self) -> u64 {
        self.work_counts[26]
    }

    pub const fn emitted_visual_runs(&self) -> u64 {
        self.work_counts[27]
    }

    pub const fn positioned_glyphs(&self) -> u64 {
        self.work_counts[28]
    }

    pub const fn emitted_carets(&self) -> u64 {
        self.work_counts[29]
    }
}

#[cfg(test)]
mod tests {
    use super::UiNativeClientTextPresentationWorkObservation;

    fn observation(work_counts: [u64; 30]) -> UiNativeClientTextPresentationWorkObservation {
        UiNativeClientTextPresentationWorkObservation::reported(
            [1, 1, 1, 1],
            work_counts,
            [[0; 32]; 4],
            0,
            [[0; 32]; 2],
            [],
            [],
            [],
        )
    }

    #[test]
    fn dpi_and_held_binding_pins_are_state_not_work() {
        let mut counts = [0; 30];
        counts[0] = 1_500;
        counts[11] = 6;
        assert!(!observation(counts).performed_work());
        for index in (1..30).filter(|index| *index != 11) {
            let mut counts = counts;
            counts[index] = 1;
            assert!(observation(counts).performed_work(), "count {index}");
        }
    }
}
