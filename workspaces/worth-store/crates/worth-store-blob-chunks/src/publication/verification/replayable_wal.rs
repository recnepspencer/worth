use super::super::{
    BlobPublicationCounterSnapshot, BlobPublicationCrashBoundaryReport, BlobPublicationDenial,
    BlobPublicationDurableWal,
};

pub(crate) fn verify_replayable_report(
    report: &BlobPublicationCrashBoundaryReport,
) -> Result<(), BlobPublicationDenial> {
    if report.replayable_durable_wal().is_some() {
        Ok(())
    } else {
        Err(BlobPublicationDenial::WalReplayEvidenceRequired {
            counters: BlobPublicationCounterSnapshot::start().with_denied_promotion(),
        })
    }
}

pub(crate) fn replayable_durable_wal(
    report: &BlobPublicationCrashBoundaryReport,
) -> Option<&BlobPublicationDurableWal> {
    report.replayable_durable_wal()
}
