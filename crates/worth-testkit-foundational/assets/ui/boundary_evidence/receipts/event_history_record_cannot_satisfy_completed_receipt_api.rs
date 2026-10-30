struct SyntheticEventHistoryRecord {
    _opaque: u64,
}

fn accepts_completed_receipt(
    _: &worth_foundational::FoundationalBoundaryEvidenceCompletedReceiptArtifact,
) {
}

fn main() {
    let history = SyntheticEventHistoryRecord { _opaque: 11 };
    accepts_completed_receipt(&history);
}
