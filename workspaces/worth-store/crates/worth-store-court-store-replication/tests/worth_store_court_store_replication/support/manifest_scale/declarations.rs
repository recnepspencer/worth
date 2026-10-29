

pub(crate) fn prove_causal_work_and_resident_reuse(observations: &[ScaleObservation]) {
    assert!(observations.iter().all(|value| {
        let point_frames = value.point_blocks.saturating_add(value.point_pages);
        value.point_work == value.point_faults.saturating_mul(2)
            && value.point_media_reads < point_frames
            && value.scan_faults <= value.scan_work
            && value.scan_work
                <= value
                    .scan_frames
                    .saturating_add(value.scan_faults.saturating_mul(2))
            && value.scan_media_reads < value.scan_frames
            && value.signal_clock_advance == 0
            && value.signal_invalidation_delta == 0
    }));
}

#[derive(Clone, Copy)]
pub(crate) struct ScaleObservation {
    pub(crate) record_count: u16,
    pub(crate) routing_level: u16,
    pub(crate) whole_blocks: u64,
    pub(crate) point_blocks: u64,
    pub(crate) point_pages: u64,
    pub(crate) point_comparisons: u64,
    pub(crate) point_work: u64,
    pub(crate) point_faults: u64,
    pub(crate) point_media_reads: u64,
    pub(crate) point_media_bytes: u64,
    pub(crate) point_manifest_bytes: u64,
    pub(crate) scan_records: u64,
    pub(crate) scan_payload_bytes: u64,
    pub(crate) scan_blocks: u64,
    pub(crate) scan_frames: u64,
    pub(crate) scan_work: u64,
    pub(crate) scan_faults: u64,
    pub(crate) scan_media_reads: u64,
    pub(crate) scan_media_bytes: u64,
    pub(crate) scan_manifest_bytes: u64,
    pub(crate) signal_clock_advance: u64,
    pub(crate) signal_invalidation_delta: u64,
    pub(crate) point_allocations: usize,
    pub(crate) scan_allocations: usize,
    pub(crate) invalid_worlds: u8,
}