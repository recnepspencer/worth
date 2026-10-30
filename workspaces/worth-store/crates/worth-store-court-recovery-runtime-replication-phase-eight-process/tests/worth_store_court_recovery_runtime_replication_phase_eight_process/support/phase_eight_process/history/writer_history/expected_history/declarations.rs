pub(crate) use std::collections::BTreeMap;
pub(crate) use std::path::Path;

pub(crate) use crate::history::schedule::in_flight_mutation::ExpectedInFlightMutation;
pub(crate) use {crate::history::parent_oracle, crate::phase_eight_process::history::schedule};

pub(crate) const RECOVERY_MEMORY_BUDGET_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedWriterHistory {
    pub(crate) seed: u64,
    pub(crate) payloads: Vec<Vec<u8>>,
    pub(crate) in_flight: ExpectedInFlightMutation,
    pub(crate) no_effect_identity: [u8; 32],
    pub(crate) durable_bindings: BTreeMap<[u8; 32], crate::phase_eight_process::history::parent_oracle::canonical_membership::ExpectedCanonicalRecord>,
    pub(crate) no_effect_idempotency: Option<[u8; 32]>,
    pub(crate) dirty_idempotency: Option<[u8; 32]>,
}

impl ExpectedWriterHistory {
    pub(crate) fn from_profile(
        schedule_seed: u64,
        operation_count: usize,
        in_flight: ExpectedInFlightMutation,
    ) -> Self {
        assert!(
            operation_count
                * crate::phase_eight_process::history::schedule::OPERATION_PAYLOAD_BYTES
                > RECOVERY_MEMORY_BUDGET_BYTES,
            "C8 fixture must exceed its admitted recovery memory budget"
        );
        let no_effect_identity = crate::phase_eight_process::history::schedule::no_effect_material(
            in_flight.perturbation_seed(),
        );
        Self {
            seed: schedule_seed,
            payloads: (0..operation_count)
                .map(|ordinal| {
                    crate::phase_eight_process::history::schedule::payload(
                        schedule_seed,
                        ordinal as u64,
                    )
                })
                .collect(),
            in_flight,
            no_effect_identity,
            durable_bindings: std::collections::BTreeMap::new(),
            no_effect_idempotency: None,
            dirty_idempotency: None,
        }
    }

    pub(crate) fn payloads(&self) -> &[Vec<u8>] {
        &self.payloads
    }

    pub(crate) fn in_flight_payload(&self) -> &[u8] {
        self.in_flight.payload()
    }

    pub(crate) const fn in_flight_material(&self) -> [u8; 32] {
        self.in_flight.material()
    }

    pub(crate) fn durable_bindings(
        &self,
    ) -> &BTreeMap<[u8; 32], crate::phase_eight_process::history::parent_oracle::canonical_membership::ExpectedCanonicalRecord>{
        &self.durable_bindings
    }

    pub(crate) fn bind_persisted_operation_identities(
        &mut self,
        root: &Path,
    ) -> Result<(), String> {
        let bindings = crate::phase_eight_process::history::parent_oracle::operation_binding::bind(
            root,
            self.seed,
            &self.payloads,
            self.no_effect_identity,
            self.in_flight_material(),
        )?;
        self.durable_bindings = bindings.durable;
        self.no_effect_idempotency = Some(bindings.no_effect);
        self.dirty_idempotency = Some(bindings.in_flight);
        Ok(())
    }

    pub(crate) const fn no_effect_idempotency(&self) -> Option<[u8; 32]> {
        self.no_effect_idempotency
    }

    pub(crate) const fn dirty_idempotency(&self) -> Option<[u8; 32]> {
        self.dirty_idempotency
    }
}
