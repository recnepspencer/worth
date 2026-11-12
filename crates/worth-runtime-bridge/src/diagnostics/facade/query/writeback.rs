use super::*;
use crate::writeback::{
    BridgeMappedWritebackFamilyInputIdentity, BridgeWritebackExecutionRecordIdentity,
    BridgeWritebackFamilyAdmissionRecordIdentity, BridgeWritebackMapperEnvelopeIdentity,
    BridgeWritebackMapperRecordIdentity, BridgeWritebackReplayRecordIdentity,
};

impl BridgeDiagnosticsFacade {
    pub fn writeback_admission_records(
        &self,
    ) -> Vec<crate::writeback::BridgeWritebackFamilyAdmissionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_admission_records()
    }

    pub fn writeback_execution_records(
        &self,
    ) -> Vec<crate::writeback::BridgeWritebackExecutionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_execution_records()
    }

    pub fn writeback_mapper_envelopes(
        &self,
    ) -> Vec<crate::writeback::BridgeWritebackMapperEnvelope> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_mapper_envelopes()
    }

    pub fn writeback_mapped_family_inputs(
        &self,
    ) -> Vec<crate::writeback::BridgeMappedWritebackFamilyInput> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_mapped_family_inputs()
    }

    pub fn writeback_mapper_records(&self) -> Vec<crate::writeback::BridgeWritebackMapperRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_mapper_records()
    }

    pub fn writeback_replay_records(&self) -> Vec<crate::writeback::BridgeWritebackReplayRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_replay_records()
    }

    pub fn last_writeback_admission_record(
        &self,
    ) -> Option<crate::writeback::BridgeWritebackFamilyAdmissionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_writeback_admission_record()
    }

    pub fn last_writeback_execution_record(
        &self,
    ) -> Result<
        crate::writeback::BridgeWritebackExecutionRecord,
        worth_foundational::facade::DiagnosticRecordAbsence,
    > {
        if !self.config.records_enabled {
            return Err(worth_foundational::facade::DiagnosticRecordAbsence::NeverRetained);
        }
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_writeback_execution_record()
    }

    pub fn last_writeback_mapper_envelope(
        &self,
    ) -> Option<crate::writeback::BridgeWritebackMapperEnvelope> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_writeback_mapper_envelope()
    }

    pub fn last_writeback_mapped_family_input(
        &self,
    ) -> Option<crate::writeback::BridgeMappedWritebackFamilyInput> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_writeback_mapped_family_input()
    }

    pub fn last_writeback_mapper_record(
        &self,
    ) -> Option<crate::writeback::BridgeWritebackMapperRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_writeback_mapper_record()
    }

    pub fn last_writeback_replay_record(
        &self,
    ) -> Result<
        crate::writeback::BridgeWritebackReplayRecord,
        worth_foundational::facade::DiagnosticRecordAbsence,
    > {
        if !self.config.records_enabled {
            return Err(worth_foundational::facade::DiagnosticRecordAbsence::NeverRetained);
        }
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_writeback_replay_record()
    }

    pub fn writeback_admission_record_for_identity(
        &self,
        record_identity: &BridgeWritebackFamilyAdmissionRecordIdentity,
    ) -> Option<crate::writeback::BridgeWritebackFamilyAdmissionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_admission_record_for_identity(record_identity)
    }

    pub fn writeback_admission_record_for_contract_digest(
        &self,
        contract_digest: &str,
    ) -> Option<crate::writeback::BridgeWritebackFamilyAdmissionRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_admission_record_for_contract_digest(contract_digest)
    }

    pub fn retained_writeback_execution_reference_for_identity(
        &self,
        record_identity: &BridgeWritebackExecutionRecordIdentity,
    ) -> Option<
        crate::diagnostics::BridgeWritebackDiagnosticReference<
            crate::writeback::BridgeWritebackExecutionRecord,
        >,
    > {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_execution_reference_for_identity(record_identity)
    }

    pub fn retained_writeback_execution_reference_for_candidate_digest(
        &self,
        candidate_digest: &str,
    ) -> Option<
        crate::diagnostics::BridgeWritebackDiagnosticReference<
            crate::writeback::BridgeWritebackExecutionRecord,
        >,
    > {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_execution_reference_for_candidate_digest(candidate_digest)
    }

    pub fn writeback_mapper_envelope_for_identity(
        &self,
        envelope_identity: &BridgeWritebackMapperEnvelopeIdentity,
    ) -> Option<crate::writeback::BridgeWritebackMapperEnvelope> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_mapper_envelope_for_identity(envelope_identity)
    }

    pub fn writeback_mapper_envelope_for_digest(
        &self,
        envelope_digest: &str,
    ) -> Option<crate::writeback::BridgeWritebackMapperEnvelope> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_mapper_envelope_for_digest(envelope_digest)
    }

    pub fn writeback_mapped_family_input_for_identity(
        &self,
        mapped_input_identity: &BridgeMappedWritebackFamilyInputIdentity,
    ) -> Option<crate::writeback::BridgeMappedWritebackFamilyInput> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_mapped_family_input_for_identity(mapped_input_identity)
    }

    pub fn writeback_mapped_family_input_for_digest(
        &self,
        mapped_input_digest: &str,
    ) -> Option<crate::writeback::BridgeMappedWritebackFamilyInput> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_mapped_family_input_for_digest(mapped_input_digest)
    }

    pub fn writeback_mapper_record_for_identity(
        &self,
        record_identity: &BridgeWritebackMapperRecordIdentity,
    ) -> Option<crate::writeback::BridgeWritebackMapperRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_mapper_record_for_identity(record_identity)
    }

    pub fn writeback_mapper_record_for_candidate_digest(
        &self,
        candidate_digest: &str,
    ) -> Option<crate::writeback::BridgeWritebackMapperRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_mapper_record_for_candidate_digest(candidate_digest)
    }

    pub fn retained_writeback_replay_reference_for_identity(
        &self,
        record_identity: &BridgeWritebackReplayRecordIdentity,
    ) -> Option<
        crate::diagnostics::BridgeWritebackDiagnosticReference<
            crate::writeback::BridgeWritebackReplayRecord,
        >,
    > {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_replay_reference_for_identity(record_identity)
    }
    pub fn writeback_execution_record_for_reference(
        &self,
        reference: &crate::diagnostics::BridgeWritebackDiagnosticReference<
            crate::writeback::BridgeWritebackExecutionRecord,
        >,
    ) -> Result<
        crate::writeback::BridgeWritebackExecutionRecord,
        worth_foundational::facade::DiagnosticRecordReadDenial,
    > {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_scope
            .read(reference)
    }
    pub fn writeback_replay_record_for_reference(
        &self,
        reference: &crate::diagnostics::BridgeWritebackDiagnosticReference<
            crate::writeback::BridgeWritebackReplayRecord,
        >,
    ) -> Result<
        crate::writeback::BridgeWritebackReplayRecord,
        worth_foundational::facade::DiagnosticRecordReadDenial,
    > {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .writeback_scope
            .read(reference)
    }
}
