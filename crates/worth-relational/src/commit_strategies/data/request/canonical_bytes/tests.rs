//! The cold footer cannot alter the original input artifact grammar or meaning.
use super::super::*;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct LegacyInput {
    schema_name: StrategyInputSchemaName,
    schema_version: StrategyInputSchemaVersion,
    canonical_bytes: Arc<[u8]>,
    digest: CanonicalStrategyInputDigest,
    artifact_name: PersistentArtifactName,
}

#[test]
fn canonical_strategy_input_preserves_prior_named_compact_wire_and_ordinary_clone() {
    for bytes in [vec![], vec![0, 255, 17, 42]] {
        let original: Arc<[u8]> = bytes.into();
        let digest = CanonicalStrategyInputDigest(Sha256::digest(&original).into());
        let legacy = LegacyInput {
            schema_name: StrategyInputSchemaName::new("input.λ"),
            schema_version: StrategyInputSchemaVersion(3),
            canonical_bytes: original.clone(),
            digest,
            artifact_name: PersistentArtifactName::new("strategy.input"),
        };
        let value = CanonicalStrategyInputArtifact::new(
            legacy.schema_name.clone(),
            legacy.schema_version,
            original,
            digest,
            legacy.artifact_name.clone(),
        );
        assert!(
            value.canonical_bytes_memory.is_none(),
            "ordinary construction allocates no grant Arc"
        );
        let clone = value.clone();
        assert!(clone.canonical_bytes_memory.is_none());
        assert_eq!(clone, value);
        assert!(Arc::ptr_eq(&clone.canonical_bytes, &value.canonical_bytes));
        assert_eq!(format!("{clone:?}"), format!("{value:?}"));
        let named = rmp_serde::to_vec_named(&legacy).unwrap();
        let compact = rmp_serde::to_vec(&legacy).unwrap();
        assert_eq!(rmp_serde::to_vec_named(&value).unwrap(), named);
        assert_eq!(rmp_serde::to_vec(&value).unwrap(), compact);
        for wire in [named, compact] {
            let decoded: CanonicalStrategyInputArtifact = rmp_serde::from_slice(&wire).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(decoded.canonical_bytes(), value.canonical_bytes());
            assert!(decoded.canonical_bytes_memory.is_none());
        }
    }
}
