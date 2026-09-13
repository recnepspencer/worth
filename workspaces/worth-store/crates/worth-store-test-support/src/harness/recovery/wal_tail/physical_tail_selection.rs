use std::path::Path;

use worth_store_recovery_physics::{
    admit_physical_wal_tail, PhysicalWalFrameFacts, PhysicalWalSegmentCandidate,
    SelectedPhysicalWalTail,
};
use worth_store_wal::{
    inspect_verified_wal_segment, prepare_wal_frame_append, WalLsnRange,
    WalSegmentArtifactIdentity, WalSegmentGeneration, WalSegmentId,
};

pub fn selected_wal_tail(range: WalLsnRange) -> SelectedPhysicalWalTail {
    let frame = prepare_wal_frame_append(
        Path::new("recovery-tail-fixture"),
        99,
        1,
        range.start().get(),
        range.end_exclusive().get(),
        "recovery-tail",
        b"payload",
    )
    .expect("recovery tail fixture frame is encodable");
    let identity = WalSegmentArtifactIdentity::new(
        WalSegmentId::new(99).expect("fixture segment id is non-zero"),
        WalSegmentGeneration::new(1).expect("fixture generation is non-zero"),
    );
    let inspection = inspect_verified_wal_segment(identity, frame.encoded_frame())
        .expect("recovery tail fixture frame is verifiable")
        .inspection();
    admit_physical_wal_tail(
        range.start().get(),
        vec![PhysicalWalSegmentCandidate::from_frame_facts(
            inspection,
            None,
            vec![PhysicalWalFrameFacts::new(range, inspection.byte_count()).unwrap()],
        )
        .unwrap()],
    )
    .expect("verified recovery tail fixture is contiguous from its frontier")
}
