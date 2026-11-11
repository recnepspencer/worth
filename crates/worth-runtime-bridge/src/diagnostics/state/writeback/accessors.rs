use super::*;

impl BridgeDiagnosticsState {
    pub(crate) fn writeback_admission_records(&self) -> Vec<BridgeWritebackFamilyAdmissionRecord> {
        self.writeback_admission_records
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn writeback_execution_records(&self) -> Vec<BridgeWritebackExecutionRecord> {
        self.writeback_execution_records
            .iter()
            .map(|record| record.value().as_ref().clone())
            .collect()
    }

    pub(crate) fn writeback_mapper_envelopes(&self) -> Vec<BridgeWritebackMapperEnvelope> {
        self.writeback_mapper_envelopes
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn writeback_mapped_family_inputs(&self) -> Vec<BridgeMappedWritebackFamilyInput> {
        self.writeback_mapped_family_inputs
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn writeback_mapper_records(&self) -> Vec<BridgeWritebackMapperRecord> {
        self.writeback_mapper_records
            .iter()
            .map(|record| (**record).clone())
            .collect()
    }

    pub(crate) fn writeback_replay_records(&self) -> Vec<BridgeWritebackReplayRecord> {
        self.writeback_replay_records
            .iter()
            .map(|record| record.value().as_ref().clone())
            .collect()
    }

    pub(crate) fn last_writeback_admission_record(
        &self,
    ) -> Option<BridgeWritebackFamilyAdmissionRecord> {
        self.writeback_admission_records
            .back()
            .map(|record| (**record).clone())
    }

    pub(crate) fn last_writeback_execution_record(
        &self,
    ) -> Result<BridgeWritebackExecutionRecord, worth_foundational::facade::DiagnosticRecordAbsence>
    {
        self.writeback_execution_records
            .back()
            .map(|record| record.value().as_ref().clone())
            .ok_or_else(|| self.writeback_scope.missing())
    }

    pub(crate) fn last_writeback_mapper_envelope(&self) -> Option<BridgeWritebackMapperEnvelope> {
        self.writeback_mapper_envelopes
            .back()
            .map(|record| (**record).clone())
    }

    pub(crate) fn last_writeback_mapped_family_input(
        &self,
    ) -> Option<BridgeMappedWritebackFamilyInput> {
        self.writeback_mapped_family_inputs
            .back()
            .map(|record| (**record).clone())
    }

    pub(crate) fn last_writeback_mapper_record(&self) -> Option<BridgeWritebackMapperRecord> {
        self.writeback_mapper_records
            .back()
            .map(|record| (**record).clone())
    }

    pub(crate) fn last_writeback_replay_record(
        &self,
    ) -> Result<BridgeWritebackReplayRecord, worth_foundational::facade::DiagnosticRecordAbsence>
    {
        self.writeback_replay_records
            .back()
            .map(|record| record.value().as_ref().clone())
            .ok_or_else(|| self.writeback_scope.missing())
    }

    pub(crate) fn writeback_admission_record_for_identity(
        &self,
        record_identity: &BridgeWritebackFamilyAdmissionRecordIdentity,
    ) -> Option<BridgeWritebackFamilyAdmissionRecord> {
        self.latest_writeback_admission_by_record_identity
            .get(record_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn writeback_admission_record_for_contract_digest(
        &self,
        contract_digest: &str,
    ) -> Option<BridgeWritebackFamilyAdmissionRecord> {
        self.latest_writeback_admission_by_contract_digest
            .get(contract_digest)
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn writeback_execution_reference_for_identity(
        &self,
        record_identity: &BridgeWritebackExecutionRecordIdentity,
    ) -> Option<
        super::super::super::writeback_retention::BridgeWritebackDiagnosticReference<
            BridgeWritebackExecutionRecord,
        >,
    > {
        self.latest_writeback_execution_by_record_identity
            .get(record_identity.as_str())
            .map(|record| record.reference(&self.writeback_scope))
    }

    pub(crate) fn writeback_execution_reference_for_candidate_digest(
        &self,
        candidate_digest: &str,
    ) -> Option<
        super::super::super::writeback_retention::BridgeWritebackDiagnosticReference<
            BridgeWritebackExecutionRecord,
        >,
    > {
        self.latest_writeback_execution_by_candidate_digest
            .get(candidate_digest)
            .map(|record| record.reference(&self.writeback_scope))
    }

    pub(crate) fn writeback_mapper_envelope_for_identity(
        &self,
        envelope_identity: &BridgeWritebackMapperEnvelopeIdentity,
    ) -> Option<BridgeWritebackMapperEnvelope> {
        self.latest_writeback_mapper_envelope_by_identity
            .get(envelope_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn writeback_mapper_envelope_for_digest(
        &self,
        envelope_digest: &str,
    ) -> Option<BridgeWritebackMapperEnvelope> {
        self.latest_writeback_mapper_envelope_by_digest
            .get(envelope_digest)
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn writeback_mapped_family_input_for_identity(
        &self,
        mapped_input_identity: &BridgeMappedWritebackFamilyInputIdentity,
    ) -> Option<BridgeMappedWritebackFamilyInput> {
        self.latest_writeback_mapped_input_by_identity
            .get(mapped_input_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn writeback_mapped_family_input_for_digest(
        &self,
        mapped_input_digest: &str,
    ) -> Option<BridgeMappedWritebackFamilyInput> {
        self.latest_writeback_mapped_input_by_digest
            .get(mapped_input_digest)
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn writeback_mapper_record_for_identity(
        &self,
        record_identity: &BridgeWritebackMapperRecordIdentity,
    ) -> Option<BridgeWritebackMapperRecord> {
        self.latest_writeback_mapper_by_record_identity
            .get(record_identity.as_str())
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn writeback_mapper_record_for_candidate_digest(
        &self,
        candidate_digest: &str,
    ) -> Option<BridgeWritebackMapperRecord> {
        self.latest_writeback_mapper_by_candidate_digest
            .get(candidate_digest)
            .cloned()
            .map(|record| (*record).clone())
    }

    pub(crate) fn writeback_replay_reference_for_identity(
        &self,
        record_identity: &BridgeWritebackReplayRecordIdentity,
    ) -> Option<
        super::super::super::writeback_retention::BridgeWritebackDiagnosticReference<
            BridgeWritebackReplayRecord,
        >,
    > {
        self.latest_writeback_replay_by_record_identity
            .get(record_identity.as_str())
            .map(|record| record.reference(&self.writeback_scope))
    }
}
