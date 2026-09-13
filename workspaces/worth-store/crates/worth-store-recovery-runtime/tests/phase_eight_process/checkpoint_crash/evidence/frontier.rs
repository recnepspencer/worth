use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_recovery_runtime::RecoveryReportOutcome;

use super::checkpoint_records::{
    assert_candidate_ending, assert_candidate_prefix, assert_complete_frontier, read_artifact,
    record_kinds,
};
use super::snapshot::DirectorySnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedCheckpointFrontier {
    outcome: RecoveryReportOutcome,
    candidate_is_residue: bool,
}

impl ExpectedCheckpointFrontier {
    pub(crate) const fn outcome(&self) -> RecoveryReportOutcome {
        self.outcome
    }

    pub(crate) const fn candidate_is_residue(&self) -> bool {
        self.candidate_is_residue
    }
}

pub(crate) fn derive_expected_frontier(stage: &str) -> ExpectedCheckpointFrontier {
    // This is the checkpoint campaign's predeclared oracle. The writer stage
    // determines whether the candidate is still residue or the namespace
    // selector has been replaced; the post-gate bytes are only evidence that
    // the declared transition actually occurred.
    let candidate_is_residue = match stage {
        "candidate-creation"
        | "candidate-append"
        | "candidate-binding-header"
        | "candidate-binding-record"
        | "candidate-footer"
        | "candidate-synchronization" => true,
        "candidate-publication" | "namespace-synchronization" => false,
        other => panic!("unrecognized checkpoint evidence stage {other}"),
    };
    ExpectedCheckpointFrontier {
        // Every named fixture retains a valid preexisting selector or
        // completes the planned publication. Recovery is therefore expected
        // to recover; any blocked result is a proof failure, not an oracle
        // branch selected from observed bytes.
        outcome: RecoveryReportOutcome::Recovered,
        candidate_is_residue,
    }
}

pub(crate) fn assert_stage_frontier(
    stage: &str,
    root: &Path,
    baseline: &DirectorySnapshot,
    effect: &DirectorySnapshot,
    frontier: &ExpectedCheckpointFrontier,
) {
    let baseline_publication = baseline.get("families/checkpoint.current");
    let effect_publication = effect.get("families/checkpoint.current");
    if frontier.candidate_is_residue {
        let (path, expected) = candidate_entry(effect)
            .unwrap_or_else(|| panic!("checkpoint stage {stage} omitted its declared candidate"));
        let bytes = read_artifact(root, &path);
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        assert_eq!(bytes.len() as u64, expected.0, "candidate length drifted");
        assert_eq!(digest, expected.1, "candidate digest drifted");
        assert_eq!(
            baseline_publication, effect_publication,
            "an unreferenced candidate must not replace the durable selector"
        );
    }
    match stage {
        "candidate-creation" => assert_candidate_prefix(root, effect, &[1]),
        "candidate-append" => assert_candidate_ending(root, effect, 2, false),
        "candidate-binding-header" => assert_candidate_ending(root, effect, 3, false),
        "candidate-binding-record" => assert_candidate_ending(root, effect, 4, false),
        "candidate-footer" | "candidate-synchronization" => {
            assert_candidate_ending(root, effect, 5, true)
        }
        "candidate-publication" | "namespace-synchronization" => {
            assert!(
                find_candidate(effect).is_none(),
                "published checkpoint left a candidate"
            );
            let effect_publication =
                effect_publication.expect("publication stage must expose a current selector");
            assert_ne!(
                Some(effect_publication),
                baseline_publication,
                "publication stage did not replace the namespace selector"
            );
            assert_complete_frontier(&record_kinds(&read_artifact(
                root,
                "families/checkpoint.current",
            )));
        }
        other => panic!("unrecognized checkpoint evidence stage {other}"),
    }
}

fn find_candidate(snapshot: &DirectorySnapshot) -> Option<&str> {
    let candidates = snapshot
        .keys()
        .filter(|path| path.starts_with("staging/") && path.ends_with(".candidate"))
        .collect::<Vec<_>>();
    assert!(
        candidates.len() <= 1,
        "checkpoint frontier exposed multiple candidate files"
    );
    candidates.first().map(|path| path.as_str())
}

fn candidate_entry(snapshot: &DirectorySnapshot) -> Option<(String, (u64, [u8; 32]))> {
    find_candidate(snapshot).map(|path| {
        (
            path.to_owned(),
            *snapshot
                .get(path)
                .expect("candidate path must have a snapshot entry"),
        )
    })
}
