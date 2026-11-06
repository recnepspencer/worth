use crate::domain_computation::convergence_epoch::tests::yielded_peer_association::*;

pub fn completed_peer(
    incumbents: &[worth_query_execution::facade::convergence_epoch::WorthQueryRetainedConvergenceCandidateEvidence],
    report: Option<
        &worth_query_execution::facade::convergence_epoch::WorthQueryBoundConvergenceReport,
    >,
) -> CompletedPeer {
    let report = report.expect("readmitted owner completion must retain its report");
    assert_eq!(incumbents.len(), 1);
    assert_eq!(
        incumbents[0].report_evidence_identity(),
        report.evidence_identity()
    );
    CompletedPeer {
        state_identity: incumbents[0].state_identity().to_owned(),
        occurrence_identity: incumbents[0].occurrence_identity().to_owned(),
    }
}
