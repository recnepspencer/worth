use std::collections::BTreeMap;
use std::path::Path;

use super::super::{parent_oracle, schedule};
use schedule::ExpectedInFlightMutation;

const RECOVERY_MEMORY_BUDGET_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedWriterHistory {
    pub(super) seed: u64,
    pub(super) payloads: Vec<Vec<u8>>,
    in_flight: ExpectedInFlightMutation,
    pub(super) no_effect_identity: [u8; 32],
    pub(super) durable_bindings: BTreeMap<[u8; 32], parent_oracle::ExpectedCanonicalRecord>,
    pub(super) no_effect_idempotency: Option<[u8; 32]>,
    pub(super) dirty_idempotency: Option<[u8; 32]>,
}

impl ExpectedWriterHistory {
    pub(crate) fn from_profile(
        schedule_seed: u64,
        operation_count: usize,
        in_flight: ExpectedInFlightMutation,
    ) -> Self {
        assert!(
            operation_count * schedule::OPERATION_PAYLOAD_BYTES > RECOVERY_MEMORY_BUDGET_BYTES,
            "C8 fixture must exceed its admitted recovery memory budget"
        );
        let no_effect_identity = schedule::no_effect_material(in_flight.perturbation_seed());
        Self {
            seed: schedule_seed,
            payloads: (0..operation_count)
                .map(|ordinal| schedule::payload(schedule_seed, ordinal as u64))
                .collect(),
            in_flight,
            no_effect_identity,
            durable_bindings: BTreeMap::new(),
            no_effect_idempotency: None,
            dirty_idempotency: None,
        }
    }

    pub(crate) fn payloads(&self) -> &[Vec<u8>] {
        &self.payloads
    }

    pub(crate) fn in_flight_identity(&self) -> [u8; 32] {
        self.in_flight.material()
    }

    pub(crate) fn in_flight_payload(&self) -> &[u8] {
        self.in_flight.payload()
    }

    pub(crate) const fn in_flight_material(&self) -> [u8; 32] {
        self.in_flight.material()
    }

    pub(crate) fn durable_bindings(
        &self,
    ) -> &BTreeMap<[u8; 32], parent_oracle::ExpectedCanonicalRecord> {
        &self.durable_bindings
    }

    pub(crate) fn bind_persisted_operation_identities(
        &mut self,
        root: &Path,
    ) -> Result<(), String> {
        let bindings = parent_oracle::bind_submitted_operations(
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
