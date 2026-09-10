//! Decode the existing flat wire schema without Serde's u128-incompatible flatten buffer.
use super::{
    ArtifactMergeAuthority, ArtifactTransitionKey, CompactChangedScopeProof,
    ContinuityAuthorityToken, MemoizedResultOrigin, OutputChange, OutputIdentity,
    ReuseBoundaryAuthority, ReuseOperationalBasis, ReuseOrigin, RuntimeArtifactHot,
    RuntimeArtifactState, RuntimeArtifactWarm, StableHashValue,
};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
struct FlatArtifactWire {
    output_hash: StableHashValue,
    #[serde(default)]
    output_change: OutputChange,
    #[serde(default)]
    recomputed: bool,
    #[serde(default)]
    dependency_count: u32,
    #[serde(default)]
    meaningful_input_changes: u32,
    #[serde(default)]
    changed_partition_count: u32,
    #[serde(default)]
    propagation_suppressed: bool,
    #[serde(default)]
    changed_scopes: CompactChangedScopeProof,
    #[serde(default)]
    output_identity: Option<OutputIdentity>,
    #[serde(default)]
    continuity_token: ContinuityAuthorityToken,
    #[serde(default)]
    memoized_origin: MemoizedResultOrigin,
    #[serde(default)]
    reuse_basis: ReuseOperationalBasis,
    #[serde(default)]
    reuse_origin: ReuseOrigin,
    #[serde(default)]
    reuse_boundary_authority: Option<ReuseBoundaryAuthority>,
    #[serde(default)]
    lineage_artifact_id: ArtifactTransitionKey,
    #[serde(default)]
    merge_authority: ArtifactMergeAuthority,
}

impl<'de> Deserialize<'de> for RuntimeArtifactState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = FlatArtifactWire::deserialize(deserializer)?;
        Ok(Self {
            hot: RuntimeArtifactHot {
                output_hash: wire.output_hash,
                output_change: wire.output_change,
                recomputed: wire.recomputed,
                dependency_count: wire.dependency_count,
                meaningful_input_changes: wire.meaningful_input_changes,
                changed_partition_count: wire.changed_partition_count,
                propagation_suppressed: wire.propagation_suppressed,
                changed_scopes: wire.changed_scopes,
            },
            warm: RuntimeArtifactWarm {
                output_identity: wire.output_identity,
                continuity_token: wire.continuity_token,
                memoized_origin: wire.memoized_origin,
                reuse_basis: wire.reuse_basis,
                reuse_origin: wire.reuse_origin,
                reuse_boundary_authority: wire.reuse_boundary_authority,
                lineage_artifact_id: wire.lineage_artifact_id,
                merge_authority: wire.merge_authority,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_flat_wire_retains_full_width_hash_and_warm_identity() {
        let artifact = RuntimeArtifactState::new(
            RuntimeArtifactHot {
                output_hash: StableHashValue::MAX,
                recomputed: true,
                ..Default::default()
            },
            RuntimeArtifactWarm {
                output_identity: Some("native identity".into()),
                ..Default::default()
            },
        );
        let wire = serde_json::to_string(&artifact).unwrap();
        let restored: RuntimeArtifactState = serde_json::from_str(&wire).unwrap();
        assert_eq!(restored, artifact);
        let value = serde_json::to_value(&artifact).unwrap();
        assert!(value.get("hot").is_none());
        assert!(value.get("warm").is_none());
        assert_eq!(
            serde_json::from_value::<RuntimeArtifactState>(value).unwrap(),
            artifact
        );
    }

    #[test]
    fn artifact_flat_wire_preserves_required_hash_defaults_and_field_validation() {
        let minimal: RuntimeArtifactState =
            serde_json::from_str(r#"{"output_hash":7,"unknown_future_field":true}"#).unwrap();
        assert_eq!(
            minimal,
            RuntimeArtifactState::new(
                RuntimeArtifactHot {
                    output_hash: 7,
                    ..Default::default()
                },
                RuntimeArtifactWarm::default(),
            )
        );
        assert!(serde_json::from_str::<RuntimeArtifactState>("{}").is_err());
        assert!(serde_json::from_str::<RuntimeArtifactState>(
            r#"{"output_hash":7,"output_hash":8}"#
        )
        .is_err());
        assert!(serde_json::from_str::<RuntimeArtifactState>(
            r#"{"output_hash":7,"recomputed":true,"recomputed":false}"#
        )
        .is_err());
    }
}
