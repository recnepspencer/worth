use super::*;

impl BridgeDiagnosticsState {
    pub(crate) fn record_writeback_admission(
        &mut self,
        record: BridgeWritebackFamilyAdmissionRecord,
        limit: usize,
    ) {
        let record = Arc::new(record);
        self.latest_writeback_admission_by_record_identity.insert(
            record.record_identity().as_str().to_string(),
            Arc::clone(&record),
        );
        self.latest_writeback_admission_by_contract_digest
            .insert(record.contract_digest().to_string(), Arc::clone(&record));
        self.writeback_admission_records.push_back(record);
        while self.writeback_admission_records.len() > limit.max(1) {
            if let Some(evicted) = self.writeback_admission_records.pop_front() {
                if self
                    .latest_writeback_admission_by_record_identity
                    .get(evicted.record_identity().as_str())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_admission_by_record_identity
                        .remove(evicted.record_identity().as_str());
                }
                if self
                    .latest_writeback_admission_by_contract_digest
                    .get(evicted.contract_digest())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_admission_by_contract_digest
                        .remove(evicted.contract_digest());
                }
            }
        }
    }

    pub(crate) fn record_writeback_mapped_input(
        &mut self,
        mapped_input: BridgeMappedWritebackFamilyInput,
        limit: usize,
    ) {
        let mapped_input = Arc::new(mapped_input);
        self.latest_writeback_mapped_input_by_identity.insert(
            mapped_input.mapped_input_identity().as_str().to_string(),
            Arc::clone(&mapped_input),
        );
        self.latest_writeback_mapped_input_by_digest
            .insert(mapped_input.digest().to_string(), Arc::clone(&mapped_input));
        self.writeback_mapped_family_inputs.push_back(mapped_input);
        while self.writeback_mapped_family_inputs.len() > limit.max(1) {
            if let Some(evicted) = self.writeback_mapped_family_inputs.pop_front() {
                if self
                    .latest_writeback_mapped_input_by_identity
                    .get(evicted.mapped_input_identity().as_str())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_mapped_input_by_identity
                        .remove(evicted.mapped_input_identity().as_str());
                }
                if self
                    .latest_writeback_mapped_input_by_digest
                    .get(evicted.digest())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_mapped_input_by_digest
                        .remove(evicted.digest());
                }
            }
        }
    }

    pub(crate) fn record_writeback_mapper_envelope(
        &mut self,
        envelope: BridgeWritebackMapperEnvelope,
        limit: usize,
    ) {
        let envelope = Arc::new(envelope);
        self.latest_writeback_mapper_envelope_by_identity.insert(
            envelope.envelope_identity().as_str().to_string(),
            Arc::clone(&envelope),
        );
        self.latest_writeback_mapper_envelope_by_digest
            .insert(envelope.digest().to_string(), Arc::clone(&envelope));
        self.writeback_mapper_envelopes.push_back(envelope);
        while self.writeback_mapper_envelopes.len() > limit.max(1) {
            if let Some(evicted) = self.writeback_mapper_envelopes.pop_front() {
                if self
                    .latest_writeback_mapper_envelope_by_identity
                    .get(evicted.envelope_identity().as_str())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_mapper_envelope_by_identity
                        .remove(evicted.envelope_identity().as_str());
                }
                if self
                    .latest_writeback_mapper_envelope_by_digest
                    .get(evicted.digest())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_mapper_envelope_by_digest
                        .remove(evicted.digest());
                }
            }
        }
    }

    pub(crate) fn record_writeback_mapper(
        &mut self,
        record: BridgeWritebackMapperRecord,
        limit: usize,
    ) {
        let record = Arc::new(record);
        self.latest_writeback_mapper_by_record_identity.insert(
            record.record_identity().as_str().to_string(),
            Arc::clone(&record),
        );
        self.latest_writeback_mapper_by_candidate_digest
            .insert(record.candidate_digest().to_string(), Arc::clone(&record));
        self.writeback_mapper_records.push_back(record);
        while self.writeback_mapper_records.len() > limit.max(1) {
            if let Some(evicted) = self.writeback_mapper_records.pop_front() {
                if self
                    .latest_writeback_mapper_by_record_identity
                    .get(evicted.record_identity().as_str())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_mapper_by_record_identity
                        .remove(evicted.record_identity().as_str());
                }
                if self
                    .latest_writeback_mapper_by_candidate_digest
                    .get(evicted.candidate_digest())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_mapper_by_candidate_digest
                        .remove(evicted.candidate_digest());
                }
            }
        }
    }

    pub(crate) fn record_writeback_execution(
        &mut self,
        record: BridgeWritebackExecutionRecord,
        limit: usize,
    ) {
        let record = Arc::new(record);
        self.latest_writeback_execution_by_record_identity.insert(
            record.record_identity().as_str().to_string(),
            Arc::clone(&record),
        );
        if let Some(candidate_digest) = record.candidate_digest() {
            self.latest_writeback_execution_by_candidate_digest
                .insert(candidate_digest.to_string(), Arc::clone(&record));
        }
        self.writeback_execution_records.push_back(record);
        while self.writeback_execution_records.len() > limit.max(1) {
            if let Some(evicted) = self.writeback_execution_records.pop_front() {
                if self
                    .latest_writeback_execution_by_record_identity
                    .get(evicted.record_identity().as_str())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_execution_by_record_identity
                        .remove(evicted.record_identity().as_str());
                }
                if let Some(candidate_digest) = evicted.candidate_digest()
                    && self
                        .latest_writeback_execution_by_candidate_digest
                        .get(candidate_digest)
                        .is_some_and(|current| Arc::ptr_eq(current, &evicted))
                {
                    self.latest_writeback_execution_by_candidate_digest
                        .remove(candidate_digest);
                }
            }
        }
    }

    pub(crate) fn annotate_last_writeback_execution_record(
        &mut self,
        execution_receipt_digest: impl Into<Arc<str>>,
    ) {
        let Some(previous_record) = self.writeback_execution_records.pop_back() else {
            return;
        };
        let previous_record = previous_record.as_ref().clone();
        self.latest_writeback_execution_by_record_identity
            .remove(previous_record.record_identity().as_str());
        if let Some(candidate_digest) = previous_record.candidate_digest()
            && self
                .latest_writeback_execution_by_candidate_digest
                .get(candidate_digest)
                .is_some_and(|current| {
                    current.record_identity() == previous_record.record_identity()
                })
        {
            self.latest_writeback_execution_by_candidate_digest
                .remove(candidate_digest);
        }

        let updated_record =
            Arc::new(previous_record.with_execution_receipt_digest(execution_receipt_digest));
        self.latest_writeback_execution_by_record_identity.insert(
            updated_record.record_identity().as_str().to_string(),
            Arc::clone(&updated_record),
        );
        if let Some(candidate_digest) = updated_record.candidate_digest() {
            self.latest_writeback_execution_by_candidate_digest
                .insert(candidate_digest.to_string(), Arc::clone(&updated_record));
        }
        self.writeback_execution_records.push_back(updated_record);
    }

    pub(crate) fn record_writeback_replay(
        &mut self,
        record: BridgeWritebackReplayRecord,
        limit: usize,
    ) {
        let record = Arc::new(record);
        self.latest_writeback_replay_by_record_identity.insert(
            record.record_identity().as_str().to_string(),
            Arc::clone(&record),
        );
        self.writeback_replay_records.push_back(record);
        while self.writeback_replay_records.len() > limit.max(1) {
            if let Some(evicted) = self.writeback_replay_records.pop_front()
                && self
                    .latest_writeback_replay_by_record_identity
                    .get(evicted.record_identity().as_str())
                    .is_some_and(|current| Arc::ptr_eq(current, &evicted))
            {
                self.latest_writeback_replay_by_record_identity
                    .remove(evicted.record_identity().as_str());
            }
        }
    }
}
