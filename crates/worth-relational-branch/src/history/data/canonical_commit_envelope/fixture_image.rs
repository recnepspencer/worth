//! Independent exact canonical test images.
#[cfg(test)]
use super::text_wire;
use super::CanonicalCommitEnvelope;

impl CanonicalCommitEnvelope {
    pub(crate) fn restore_fixture_custody(&self, copied: &mut Self) {
        // The embedded wire omits the derived roster. These snapshots retain
        // their actual owner and existing exact-payload index verification.
        copied.body.derived_index_artifacts = self.body.derived_index_artifacts.clone();
        assert_eq!(
            &*copied, self,
            "a copied fixture preserves the original canonical meaning"
        );
        copied.body.authored_index_writes = self.body.authored_index_writes.clone();
        if let Some(original) = self.recorded_original_inputs() {
            copied
                .retain_recorded_original_inputs(original.clone())
                .expect("a copied fixture retains its source's exact recorded inputs");
        }
    }

    /// Independent branch-predicate input. Construct the second image before
    /// the operation under test and leave its fixture allocations outside the
    /// operation's request account.
    #[cfg(test)]
    pub(crate) fn independent_fixture_image(&self) -> Self {
        let bytes = rmp_serde::to_vec_named(
            &crate::history::data::EmbeddedCheckpointEnvelopeRef::new(self),
        )
        .expect("the actual original checkpoint projection encodes");
        let wire: text_wire::CanonicalEnvelopeWire<serde::de::IgnoredAny> =
            rmp_serde::from_slice(&bytes).expect("a valid fixture decodes its own wire");
        // The independent body keeps the already-admitted meanings of its
        // exact source dictionary. Its caller does not request readmission.
        let mut copied = wire
            .into_envelope(self.body.symbol_texts.clone())
            .expect("the original dictionary names every copied symbol");
        self.restore_fixture_custody(&mut copied);
        copied
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait CanonicalCommitEnvelopeFixturePort: Sized {
    fn restore_fixture_custody_for_test(&self, copied: &mut Self);
}

impl CanonicalCommitEnvelopeFixturePort for CanonicalCommitEnvelope {
    fn restore_fixture_custody_for_test(&self, copied: &mut Self) {
        self.restore_fixture_custody(copied)
    }
}
