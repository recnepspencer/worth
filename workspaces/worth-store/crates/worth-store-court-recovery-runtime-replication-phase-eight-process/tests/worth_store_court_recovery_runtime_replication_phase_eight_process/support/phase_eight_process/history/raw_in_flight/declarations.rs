pub(crate) use {std::path::Path};

pub(crate) use {crate::phase_eight_process::history::artifacts::collect_files};
pub(crate) use {crate::history::parent_oracle, crate::phase_eight_process::history::writer_history::expected_history::ExpectedWriterHistory};

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
        let selected_basis = crate::phase_eight_process::history::parent_oracle::selected_basis::select(&files)?;
        let idempotency = self
            .dirty_idempotency()
            .ok_or_else(|| "missing persisted dirty-operation identity binding".to_owned())?;
        let (identity_present, payload_present) = crate::history::parent_oracle::in_flight::classify(
            &selected_basis,
            &idempotency,
            self.in_flight_payload(),
        )?;
        let current_root_payloads = crate::phase_eight_process::history::parent_oracle::canonical_membership::current_root_payloads(&files)?;
        Ok(
            if current_root_payloads.contains(self.in_flight_payload()) {
                crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::DurableEffect
            } else {
                classify_presence(identity_present, payload_present)
            },
        )
    }
}

pub(crate) fn classify_presence(identity_present: bool, payload_present: bool) -> InFlightMutationFate {
    match (identity_present, payload_present) {
        (_, true) => crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::DurableEffect,
        // A missing identity and payload is only missing evidence. The crash
        // contract forbids turning non-observation into a no-effect
        // conclusion; a persisted terminal cancellation fact is required for
        // ProvenNoEffect.
        (false, false) => crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::Indeterminate,
        (true, false) => crate::phase_eight_process::history::raw_in_flight::InFlightMutationFate::Indeterminate,
    }
}