//! Deterministic counts of the work behind one presentation attempt.
//!
//! Wall-clock timings on a loaded machine swing from run to run; these counts
//! do not. Each stage that walks glyph records reports how many it touched,
//! and the digested bytes and hash-map inserts are counted over the whole
//! attempt. The counts live in thread-local cells, because one thread presents
//! and each parallel test sees only its own work. Allocations, when the binary
//! counts them, are the exception: the process counts them, so they include
//! other threads' and are a diagnostic, not a deterministic count.
//!
//! Everything the presenting thread counts between two takes is one frame's,
//! so work outside presentation proper, such as a Query idempotency key
//! digested on that thread, is counted with the frame it falls in.

use std::cell::Cell;
use std::sync::OnceLock;

use sha2::digest::{consts::U32, FixedOutput, HashMarker, Output, OutputSizeUser, Update};

/// A stage of presentation that touches glyph records.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiPresentationWorkStage {
    /// Deriving raster demand from qualified layouts, and rebuilding the
    /// glyph runs a paint-only change keeps.
    DemandDerivation,
    /// Mapping demand records into the request basis.
    RequestBasis,
    /// Normalizing demand into atlas requests.
    AtlasNormalize,
    /// Planning atlas placement for the requests.
    AtlasPlan,
    /// Settling planned entries into the atlas.
    AtlasSettle,
    /// Pinning and releasing atlas entries.
    Pins,
    /// Admitting glyph runs against their demand and commands.
    GlyphRunAdmission,
    /// Computing foreground coverage at a target extent, and staging it for
    /// retained replay.
    CoverageStaging,
    /// Planning native glyph commands from runs.
    CommandPlanning,
    /// Encoding glyph vertices.
    VertexEncode,
}

const STAGES: usize = 10;

impl UiPresentationWorkStage {
    /// Every stage, in presentation order.
    pub const ALL: [Self; STAGES] = [
        Self::DemandDerivation,
        Self::RequestBasis,
        Self::AtlasNormalize,
        Self::AtlasPlan,
        Self::AtlasSettle,
        Self::Pins,
        Self::GlyphRunAdmission,
        Self::CoverageStaging,
        Self::CommandPlanning,
        Self::VertexEncode,
    ];

    /// The stage's name in the resize trace.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::DemandDerivation => "demand",
            Self::RequestBasis => "basis",
            Self::AtlasNormalize => "normalize",
            Self::AtlasPlan => "plan",
            Self::AtlasSettle => "settle",
            Self::Pins => "pins",
            Self::GlyphRunAdmission => "admission",
            Self::CoverageStaging => "coverage",
            Self::CommandPlanning => "commands",
            Self::VertexEncode => "vertices",
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

/// The work one presentation attempt performed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiPresentationWorkCounts {
    glyphs: [u64; STAGES],
    digested_bytes: u64,
    map_inserts: u64,
    allocations: Option<u64>,
}

impl UiPresentationWorkCounts {
    /// Glyph records `stage` touched.
    #[must_use]
    pub const fn glyphs(self, stage: UiPresentationWorkStage) -> u64 {
        self.glyphs[stage.index()]
    }

    /// Glyph records every stage touched, together.
    #[must_use]
    pub fn glyphs_total(self) -> u64 {
        self.glyphs
            .iter()
            .fold(0, |total, glyphs| total.saturating_add(*glyphs))
    }

    /// Bytes fed to SHA-256.
    #[must_use]
    pub const fn digested_bytes(self) -> u64 {
        self.digested_bytes
    }

    /// Entries inserted into the hash maps and sets that index glyph work:
    /// demand keys, atlas candidates and commits, pins, uploads, and run
    /// admission. Maps outside the glyph path are not counted.
    #[must_use]
    pub const fn map_inserts(self) -> u64 {
        self.map_inserts
    }

    /// Heap allocations and reallocations the whole process made since the
    /// previous take on this thread, when the process counts them. Other
    /// threads' are included, so this is a diagnostic, not a deterministic
    /// count.
    #[must_use]
    pub const fn allocations(self) -> Option<u64> {
        self.allocations
    }
}

thread_local! {
    static GLYPHS: [Cell<u64>; STAGES] = const { [const { Cell::new(0) }; STAGES] };
    static DIGESTED_BYTES: Cell<u64> = const { Cell::new(0) };
    static MAP_INSERTS: Cell<u64> = const { Cell::new(0) };
    static ALLOCATIONS_BEFORE: Cell<Option<u64>> = const { Cell::new(None) };
}

static ALLOCATION_COUNTER: OnceLock<fn() -> u64> = OnceLock::new();

fn add(cell: &Cell<u64>, count: usize) {
    cell.set(
        cell.get()
            .saturating_add(u64::try_from(count).unwrap_or(u64::MAX)),
    );
}

/// Records that `stage` touched `glyphs` glyph records.
pub fn record_presentation_glyphs(stage: UiPresentationWorkStage, glyphs: usize) {
    GLYPHS.with(|cells| add(&cells[stage.index()], glyphs));
}

/// Records `bytes` fed to SHA-256. Only [`UiCountedSha256`] digests, so only
/// it records them.
fn record_presentation_digest(bytes: usize) {
    DIGESTED_BYTES.with(|cell| add(cell, bytes));
}

/// Records `inserts` entries added to a hash map or set.
pub fn record_presentation_map_inserts(inserts: usize) {
    MAP_INSERTS.with(|cell| add(cell, inserts));
}

/// Installs the process's allocation counter: a reading of every heap
/// allocation so far. The first counter installed stays; this reports
/// whether `counter` is it.
pub fn install_presentation_allocation_counter(counter: fn() -> u64) -> bool {
    ALLOCATION_COUNTER.set(counter).is_ok()
}

/// Returns the work counted since the last call on this thread and starts
/// counting afresh.
pub fn take_presentation_work() -> UiPresentationWorkCounts {
    let glyphs = GLYPHS.with(|cells| std::array::from_fn(|stage| cells[stage].replace(0)));
    let allocations = ALLOCATION_COUNTER.get().map(|counter| counter());
    let before = ALLOCATIONS_BEFORE.with(|cell| cell.replace(allocations));
    UiPresentationWorkCounts {
        glyphs,
        digested_bytes: DIGESTED_BYTES.with(|cell| cell.replace(0)),
        map_inserts: MAP_INSERTS.with(|cell| cell.replace(0)),
        allocations: allocations
            .zip(before)
            .map(|(now, before)| now.saturating_sub(before)),
    }
}

/// SHA-256 that counts the bytes it digests into the presentation work.
/// Its digests are exactly SHA-256's.
#[derive(Clone, Default)]
pub struct UiCountedSha256(sha2::Sha256);

impl HashMarker for UiCountedSha256 {}

impl OutputSizeUser for UiCountedSha256 {
    type OutputSize = U32;
}

impl Update for UiCountedSha256 {
    fn update(&mut self, data: &[u8]) {
        record_presentation_digest(data.len());
        Update::update(&mut self.0, data);
    }
}

impl FixedOutput for UiCountedSha256 {
    fn finalize_into(self, out: &mut Output<Self>) {
        FixedOutput::finalize_into(self.0, out);
    }
}

#[cfg(test)]
mod tests {
    use sha2::Digest;

    use super::{
        record_presentation_glyphs, record_presentation_map_inserts, take_presentation_work,
        UiCountedSha256, UiPresentationWorkCounts, UiPresentationWorkStage,
    };

    #[test]
    fn counted_digests_match_sha256_and_count_their_bytes() {
        let _ = take_presentation_work();
        let mut counted = UiCountedSha256::new();
        counted.update(b"worth");
        counted.update([0_u8; 11]);
        let mut plain = sha2::Sha256::new();
        plain.update(b"worth");
        plain.update([0_u8; 11]);
        assert_eq!(counted.finalize(), plain.finalize());
        assert_eq!(take_presentation_work().digested_bytes(), 16);
    }

    #[test]
    fn taking_the_work_starts_counting_afresh() {
        let _ = take_presentation_work();
        record_presentation_glyphs(UiPresentationWorkStage::AtlasPlan, 7);
        record_presentation_glyphs(UiPresentationWorkStage::AtlasPlan, 5);
        record_presentation_glyphs(UiPresentationWorkStage::Pins, 2);
        record_presentation_map_inserts(3);
        let work = take_presentation_work();
        assert_eq!(work.glyphs(UiPresentationWorkStage::AtlasPlan), 12);
        assert_eq!(work.glyphs(UiPresentationWorkStage::Pins), 2);
        assert_eq!(work.glyphs_total(), 14);
        assert_eq!(work.map_inserts(), 3);
        assert_eq!(
            take_presentation_work(),
            UiPresentationWorkCounts::default()
        );
    }
}
