use crate::source::{
    SourceFailureRecord, SourceFailureRecordIdentity, SourceMaterializationRecord,
    SourceMaterializationRecordIdentity,
};

use super::*;

impl BridgeDiagnosticsFacade {
    pub fn source_materialization_records(&self) -> Vec<SourceMaterializationRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .source_materialization_records()
    }

    pub fn source_failure_records(&self) -> Vec<SourceFailureRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .source_failure_records()
    }

    pub fn last_source_materialization_record(&self) -> Option<SourceMaterializationRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_source_materialization_record()
    }

    pub fn last_source_failure_record(&self) -> Option<SourceFailureRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .last_source_failure_record()
    }

    pub fn source_materialization_record_for_identity(
        &self,
        record_identity: &SourceMaterializationRecordIdentity,
    ) -> Option<SourceMaterializationRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .source_materialization_record_for_identity(record_identity)
    }

    pub fn source_failure_for_declaration_identity(
        &self,
        declaration_identity: &str,
    ) -> Option<SourceFailureRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .source_failure_for_declaration_identity(declaration_identity)
    }

    pub fn source_failure_record_for_identity(
        &self,
        failure_identity: &SourceFailureRecordIdentity,
    ) -> Option<SourceFailureRecord> {
        self.state
            .read()
            .expect("bridge diagnostics lock poisoned")
            .source_failure_record_for_identity(failure_identity)
    }
}
