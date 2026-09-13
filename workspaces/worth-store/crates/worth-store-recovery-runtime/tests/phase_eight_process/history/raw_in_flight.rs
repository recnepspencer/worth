use std::path::Path;

use super::artifacts::collect_files;
use super::{parent_oracle, ExpectedWriterHistory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InFlightMutationFate {
    DurableEffect,
    Indeterminate,
}

impl ExpectedWriterHistory {
    pub(crate) fn classify_in_flight_from_physical_artifacts(
        &self,
        root: &Path,
    ) -> Result<InFlightMutationFate, String> {
        let root = root
            .canonicalize()
            .map_err(|error| format!("canonicalize semantic in-flight root: {error}"))?;
        let mut files = Vec::new();
        collect_files(&root, &root, &mut files)?;
        let (identity_present, payload_present) = parent_oracle::classify_in_flight_artifacts(
            &files,
            &self.in_flight_identity(),
            self.in_flight_payload(),
        )?;
        let current_root_payloads = parent_oracle::current_root_payloads(&files)?;
        Ok(
            if current_root_payloads.contains(self.in_flight_payload()) {
                InFlightMutationFate::DurableEffect
            } else {
                classify_presence(identity_present, payload_present)
            },
        )
    }
}

fn classify_presence(identity_present: bool, payload_present: bool) -> InFlightMutationFate {
    match (identity_present, payload_present) {
        (_, true) => InFlightMutationFate::DurableEffect,
        // A missing identity and payload is only missing evidence. The crash
        // contract forbids turning non-observation into a no-effect
        // conclusion; a persisted terminal cancellation fact is required for
        // ProvenNoEffect.
        (false, false) => InFlightMutationFate::Indeterminate,
        (true, false) => InFlightMutationFate::Indeterminate,
    }
}
