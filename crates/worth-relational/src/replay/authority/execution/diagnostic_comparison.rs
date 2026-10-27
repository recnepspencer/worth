//! Replay compares the diagnostic authority actually retained by its source.
use super::super::surface_comparison::{compare_replay_surface, surface_basis_for_diagnostics};
use crate::diagnostics::data::RelationalDiagnosticArtifact;
use crate::replay::data::{
    CanonicalCommitEnvelope, ReplayMismatch, ReplayMismatchClass, ReplayObservableSurface,
    ReplayVerificationPlan,
};
use crate::runtime::RelationalRuntime;

pub(super) fn compare(
    runtime: &RelationalRuntime,
    plan: &ReplayVerificationPlan,
    mismatches: &mut Vec<ReplayMismatch>,
    source: &CanonicalCommitEnvelope,
    reconstructed: &CanonicalCommitEnvelope,
) {
    let journal_backed = runtime
        .history
        .commit_artifact(source.commit.commit_id)
        .is_some_and(|artifact| artifact.envelope_residency().journal_index().is_some());
    // RCCA authenticates these headers and deliberately omits capture entries.
    // Captured memory envelopes retain their full comparison, even when empty.
    let expected_projection;
    let observed_projection;
    let (expected, observed) = if journal_backed {
        expected_projection = journal_projection(&source.diagnostics_summary);
        observed_projection = journal_projection(&reconstructed.diagnostics_summary);
        (&expected_projection, &observed_projection)
    } else {
        (
            &source.diagnostics_summary,
            &reconstructed.diagnostics_summary,
        )
    };
    compare_replay_surface(
        runtime,
        plan,
        mismatches,
        ReplayObservableSurface::Diagnostics,
        ReplayMismatchClass::DiagnosticsDrift,
        surface_basis_for_diagnostics(expected),
        surface_basis_for_diagnostics(observed),
        "retained diagnostic authority differed",
        || expected == observed,
        || format!("{expected:?}"),
        || format!("{observed:?}"),
    );
}

fn journal_projection(summary: &RelationalDiagnosticArtifact) -> RelationalDiagnosticArtifact {
    RelationalDiagnosticArtifact {
        scope: summary.scope,
        kind: summary.kind,
        determinism: summary.determinism,
        entries: Vec::new(),
    }
}
