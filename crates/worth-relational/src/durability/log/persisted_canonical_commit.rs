use serde::{Deserialize, Deserializer, Serialize};

use crate::history::data::{
    CanonicalCommitEnvelope, CheckpointCanonicalEnvelopeRef, PositionedCanonicalCommit,
};
use crate::publication::patch::data::PatchStreamPosition;

pub(super) const CANONICAL_PROTOCOL_VERSION: u16 = 5;

/// Raw native-file vocabulary. Decoding this type never grants current
/// canonical authority; callers must pass it through owner readmission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PersistedCanonicalCommit {
    /// Version of this native commit entry, independent of domain schema IDs.
    /// Only the current format is readmitted; missing/prior versions refuse.
    #[serde(default)]
    canonical_protocol_version: u16,
    position: PatchStreamPosition,
    #[serde(rename = "canonical_v5")]
    canonical: CanonicalCommitEnvelope,
}

impl<'de> Deserialize<'de> for PersistedCanonicalCommit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(default)]
            canonical_protocol_version: u16,
            position: PatchStreamPosition,
            #[serde(default)]
            canonical: Option<CanonicalCommitEnvelope>,
            #[serde(default)]
            canonical_v5: Option<CanonicalCommitEnvelope>,
        }
        let Wire {
            canonical_protocol_version,
            position,
            canonical,
            canonical_v5,
        } = Wire::deserialize(deserializer)?;
        let canonical = match (canonical_protocol_version, canonical, canonical_v5) {
            (CANONICAL_PROTOCOL_VERSION, None, Some(current)) => current,
            _ => {
                return Err(serde::de::Error::custom(
                    "unsupported native canonical format; v5 required",
                ))
            }
        };
        if !canonical.codec_retirement_headers_valid() {
            return Err(serde::de::Error::custom(
                "invalid complete native codec disposition header",
            ));
        }
        Ok(Self {
            canonical_protocol_version,
            position,
            canonical,
        })
    }
}

/// Borrowed checkpoint encoding of a canonical envelope. The derived index
/// cache is omitted from the wire without copying its authoritative body.
#[derive(Serialize)]
pub(super) struct PersistedCheckpointCommitRef<'a> {
    canonical_protocol_version: u16,
    position: PatchStreamPosition,
    #[serde(rename = "canonical_v5")]
    canonical: CheckpointCanonicalEnvelopeRef<'a>,
}

impl<'a> PersistedCheckpointCommitRef<'a> {
    pub(super) fn from_positioned(commit: &'a PositionedCanonicalCommit) -> Self {
        Self {
            canonical_protocol_version: CANONICAL_PROTOCOL_VERSION,
            position: commit.position(),
            canonical: CheckpointCanonicalEnvelopeRef::new(commit.envelope()),
        }
    }
}

impl PersistedCanonicalCommit {
    #[cfg(test)]
    pub(crate) fn from_positioned(commit: &PositionedCanonicalCommit) -> Self {
        Self {
            canonical_protocol_version: CANONICAL_PROTOCOL_VERSION,
            position: commit.position(),
            canonical: commit.envelope().clone(),
        }
    }

    pub(crate) fn from_parts(
        position: PatchStreamPosition,
        canonical: CanonicalCommitEnvelope,
    ) -> Self {
        Self {
            canonical_protocol_version: CANONICAL_PROTOCOL_VERSION,
            position,
            canonical,
        }
    }

    /// Checkpoint envelopes carry canonical history, not rebuildable index
    /// caches. The versioned checkpoint artifact carries retained generations.
    #[cfg(test)]
    pub(crate) fn from_checkpoint_positioned(commit: &PositionedCanonicalCommit) -> Self {
        let mut persisted = Self::from_positioned(commit);
        persisted.canonical.derived_index_artifacts = Default::default();
        persisted
    }

    #[cfg(test)]
    pub(crate) fn envelope_mut_for_test(&mut self) -> &mut CanonicalCommitEnvelope {
        &mut self.canonical
    }

    pub(crate) fn readmit(
        self,
    ) -> Result<crate::durability::migration::ReadmittedCanonicalCommit, String> {
        if self.canonical_protocol_version != CANONICAL_PROTOCOL_VERSION {
            return Err("unsupported canonical commit protocol version".into());
        }
        crate::durability::migration::ReadmittedCanonicalCommit::readmit_current(
            self.position,
            self.canonical,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde::{Deserialize, Serialize};

    use super::{
        PersistedCanonicalCommit, PersistedCheckpointCommitRef, CANONICAL_PROTOCOL_VERSION,
    };
    use crate::history::data::{CheckpointCanonicalEnvelopeRef, PositionedCanonicalCommit};
    use crate::publication::patch::data::PatchStreamPosition;
    use crate::tests::support::{create_entity_outcome, persisted_runtime_with_test_schema};

    #[test]
    fn current_segment_and_checkpoint_readmit_exact_touches_and_refuse_prior_format() {
        let host_request = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        let runtime = persisted_runtime_with_test_schema();
        let committed = create_entity_outcome(execution, &runtime, "touch-wire");
        let envelope = runtime
            .replay()
            .canonical_commit_envelope(committed.commit.commit_id)
            .expect("performed commit has canonical authority")
            .clone();
        let positioned =
            PositionedCanonicalCommit::for_test(PatchStreamPosition(1), Arc::new(envelope));
        let persisted = PersistedCanonicalCommit::from_positioned(&positioned);
        #[derive(Deserialize)]
        struct PriorReader {
            position: PatchStreamPosition,
            canonical: crate::history::data::CanonicalCommitEnvelope,
        }
        for bytes in [
            rmp_serde::to_vec_named(&persisted).expect("segment map"),
            rmp_serde::to_vec_named(&PersistedCheckpointCommitRef::from_positioned(&positioned))
                .expect("checkpoint map"),
        ] {
            assert!(
                rmp_serde::from_slice::<PriorReader>(&bytes).is_err(),
                "prior readers must reject current format rather than drop native meaning"
            );
            let decoded: PersistedCanonicalCommit =
                rmp_serde::from_slice(&bytes).expect("native commit map decodes");
            assert_eq!(decoded.canonical_protocol_version, 5);
            let readmitted = decoded
                .readmit()
                .expect("current native commit is admitted");
            assert!(readmitted
                .envelope()
                .descriptive_touches()
                .exact_touches()
                .is_some());
        }

        #[derive(Serialize)]
        struct InterimV5Ref<'a> {
            canonical_protocol_version: u16,
            position: PatchStreamPosition,
            canonical_v5: CheckpointCanonicalEnvelopeRef<'a>,
        }
        let incomplete = rmp_serde::to_vec_named(&InterimV5Ref {
            canonical_protocol_version: CANONICAL_PROTOCOL_VERSION,
            position: positioned.position(),
            canonical_v5: CheckpointCanonicalEnvelopeRef::new(positioned.envelope())
                .without_codec_retirements_for_test(),
        })
        .unwrap();
        let denied = rmp_serde::from_slice::<PersistedCanonicalCommit>(&incomplete).unwrap_err();
        assert!(
            denied
                .to_string()
                .contains("required_output_codec_retirements"),
            "{denied}"
        );
        let denied =
            rmp_serde::from_slice::<super::text_wire::CanonicalCommitWire<'_>>(&incomplete)
                .err()
                .expect("borrowed current reader must reject incomplete interim v5");
        assert!(
            denied
                .to_string()
                .contains("required_output_codec_retirements"),
            "{denied}"
        );

        #[derive(Serialize)]
        struct LegacyPersistedRef<'a> {
            position: PatchStreamPosition,
            canonical: CheckpointCanonicalEnvelopeRef<'a>,
        }
        let old_wire = rmp_serde::to_vec_named(&LegacyPersistedRef {
            position: positioned.position(),
            canonical: CheckpointCanonicalEnvelopeRef::legacy_without_touches_for_test(
                positioned.envelope(),
            ),
        })
        .expect("prior named MessagePack checkpoint map");
        let prior: PriorReader = rmp_serde::from_slice(&old_wire).expect("prior reader decodes v2");
        assert_eq!(prior.position, positioned.position());
        assert_eq!(prior.canonical.commit, positioned.envelope().commit);
        assert!(
            rmp_serde::from_slice::<PersistedCanonicalCommit>(&old_wire).is_err(),
            "old native text formats are unsupported"
        );

        #[derive(Serialize)]
        struct MismatchedRef<'a> {
            canonical_protocol_version: u16,
            position: PatchStreamPosition,
            canonical: CheckpointCanonicalEnvelopeRef<'a>,
        }
        let v3_with_legacy_field = rmp_serde::to_vec_named(&MismatchedRef {
            canonical_protocol_version: CANONICAL_PROTOCOL_VERSION,
            position: positioned.position(),
            canonical: CheckpointCanonicalEnvelopeRef::new(positioned.envelope()),
        })
        .expect("mismatched native map");
        assert!(rmp_serde::from_slice::<PersistedCanonicalCommit>(&v3_with_legacy_field).is_err());

        #[derive(Serialize)]
        struct DualRef<'a> {
            canonical_protocol_version: u16,
            position: PatchStreamPosition,
            canonical: CheckpointCanonicalEnvelopeRef<'a>,
            canonical_v5: CheckpointCanonicalEnvelopeRef<'a>,
        }
        let dual = rmp_serde::to_vec_named(&DualRef {
            canonical_protocol_version: CANONICAL_PROTOCOL_VERSION,
            position: positioned.position(),
            canonical: CheckpointCanonicalEnvelopeRef::new(positioned.envelope()),
            canonical_v5: CheckpointCanonicalEnvelopeRef::new(positioned.envelope()),
        })
        .expect("dual native map");
        assert!(rmp_serde::from_slice::<PersistedCanonicalCommit>(&dual).is_err());

        #[derive(Serialize)]
        struct V2WithV3FieldRef<'a> {
            canonical_protocol_version: u16,
            position: PatchStreamPosition,
            canonical_v5: CheckpointCanonicalEnvelopeRef<'a>,
        }
        let v2_with_v3_field = rmp_serde::to_vec_named(&V2WithV3FieldRef {
            canonical_protocol_version: 2,
            position: positioned.position(),
            canonical_v5: CheckpointCanonicalEnvelopeRef::new(positioned.envelope()),
        })
        .expect("mismatched legacy map");
        assert!(rmp_serde::from_slice::<PersistedCanonicalCommit>(&v2_with_v3_field).is_err());

        let mut forged = persisted.clone();
        forged.canonical_protocol_version = 4;
        assert!(
            forged.readmit().is_err(),
            "v4 cannot claim current producer succession authority"
        );
    }
}

mod text_wire;
pub(crate) use text_wire::CanonicalCommitWire;
