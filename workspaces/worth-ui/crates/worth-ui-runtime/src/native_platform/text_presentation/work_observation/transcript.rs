use sha2::{Digest, Sha256};

use super::super::UiNativeTextPresentationPrepared;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiNativeTextTranscriptEvidence {
    layout_set: [u8; 32],
    raster_key_set: [u8; 32],
    glyph_runs: [u8; 32],
    intrinsic_glyph_runs_digest: [u8; 32],
    intrinsic_glyph_runs: u64,
}

impl UiNativeTextTranscriptEvidence {
    pub(super) fn from_prepared(prepared: &UiNativeTextPresentationPrepared) -> Self {
        let mut layouts = prepared
            .glyph_runs()
            .iter()
            .map(|run| run.layout_identity().digest().to_vec())
            .collect::<Vec<_>>();
        let mut raster_keys = prepared
            .glyph_runs()
            .iter()
            .map(|run| run.raster_key().canonical_evidence_bytes().to_vec())
            .collect::<Vec<_>>();
        let mut glyph_runs = prepared
            .glyph_runs()
            .iter()
            .map(|run| run.canonical_transcript_bytes())
            .collect::<Vec<_>>();
        let mut intrinsic = prepared
            .glyph_runs()
            .iter()
            .filter(|run| {
                matches!(
                    run.raster_key().source(),
                    worth_ui_host_contract::UiGlyphRasterSource::ColorOutline
                        | worth_ui_host_contract::UiGlyphRasterSource::ColorBitmap
                )
            })
            .map(|run| run.canonical_transcript_bytes())
            .collect::<Vec<_>>();
        let intrinsic_glyph_runs = intrinsic.len() as u64;
        Self {
            layout_set: digest_set(&mut layouts),
            raster_key_set: digest_set(&mut raster_keys),
            glyph_runs: digest_rows(&mut glyph_runs),
            intrinsic_glyph_runs_digest: digest_rows(&mut intrinsic),
            intrinsic_glyph_runs,
        }
    }

    pub(super) const fn digests(self) -> [[u8; 32]; 4] {
        [
            self.layout_set,
            self.raster_key_set,
            self.glyph_runs,
            self.intrinsic_glyph_runs_digest,
        ]
    }

    pub(super) const fn intrinsic_glyph_runs(self) -> u64 {
        self.intrinsic_glyph_runs
    }
}

fn digest_set(rows: &mut Vec<Vec<u8>>) -> [u8; 32] {
    rows.sort_unstable();
    rows.dedup();
    digest_ordered(rows)
}

fn digest_rows(rows: &mut [Vec<u8>]) -> [u8; 32] {
    let mut row_hashes = rows
        .iter()
        .map(|row| Sha256::digest(row).to_vec())
        .collect::<Vec<_>>();
    row_hashes.sort_unstable();
    digest_ordered(&row_hashes)
}

fn digest_ordered(rows: &[Vec<u8>]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update((rows.len() as u64).to_le_bytes());
    for row in rows {
        digest.update((row.len() as u64).to_le_bytes());
        digest.update(row);
    }
    digest.finalize().into()
}
