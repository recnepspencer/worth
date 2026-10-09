use super::{streaming_policy_digest, ResourceRetentionCompactionPolicyProvenanceDigestBasis};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct PreviousOwnedBasis {
    schema_version: &'static str,
    retained_history_decision_digests: Vec<String>,
    retry_lineage_decision_digests: Vec<String>,
}

#[test]
fn streamed_policy_provenance_matches_canonical_json_for_empty_multiple_and_escaped_inputs() {
    for (history, retry) in [
        (vec![], vec![]),
        (vec!["alpha", "beta", "gamma"], vec!["retry-1", "retry-2"]),
        (
            vec!["quoted \"policy\"", "\u{2603}\\newline\n"],
            vec!["é", "\u{0001}"],
        ),
    ] {
        let basis = ResourceRetentionCompactionPolicyProvenanceDigestBasis {
            schema_version: "worth.resource.retention-compaction-policy-provenance.v1",
            retained_history_decision_digests: &history,
            retry_lineage_decision_digests: &retry,
            expired_lifecycle: Default::default(),
            expired_retry: Default::default(),
        };
        let previous_bytes = serde_json::to_vec(&PreviousOwnedBasis {
            schema_version: basis.schema_version,
            retained_history_decision_digests: history.iter().map(ToString::to_string).collect(),
            retry_lineage_decision_digests: retry.iter().map(ToString::to_string).collect(),
        })
        .unwrap();
        assert_eq!(serde_json::to_vec(&basis).unwrap(), previous_bytes);
        let expected = Sha256::digest(previous_bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(streaming_policy_digest(&basis), expected);
    }
}
