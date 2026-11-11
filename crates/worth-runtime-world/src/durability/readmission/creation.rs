//! Retain exact creation subjects and positions without granting component fate.
use crate::durability::artifact::{WorldBranchCreatedRecord, WorldCreationIntentRecord};
use crate::durability::{DecodedWorldRecord, WorldRecord};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldCreationIntent {
    record: Arc<DecodedWorldRecord>,
    position: u64,
    digest: [u8; 32],
}
impl AdmittedWorldCreationIntent {
    pub(super) fn from_validated_journal(
        record: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        assert!(matches!(&record.record, WorldRecord::CreationIntent(_)));
        Self {
            record,
            position,
            digest,
        }
    }
    pub(crate) fn intent(&self) -> &WorldCreationIntentRecord {
        let WorldRecord::CreationIntent(intent) = &self.record.record else {
            unreachable!("admitted intent class")
        };
        intent
    }
    pub(crate) fn record(&self) -> &Arc<DecodedWorldRecord> {
        &self.record
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

/// Frame admission precedes dependency and actual component-owner admission.
#[derive(Clone, Debug)]
pub(crate) struct RecordedWorldBranchCreated {
    record: Arc<DecodedWorldRecord>,
    position: u64,
    digest: [u8; 32],
}
impl RecordedWorldBranchCreated {
    pub(super) fn from_validated_journal(
        record: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        assert!(matches!(&record.record, WorldRecord::BranchCreated(_)));
        Self {
            record,
            position,
            digest,
        }
    }
    pub(crate) fn created(&self) -> &WorldBranchCreatedRecord {
        let WorldRecord::BranchCreated(created) = &self.record.record else {
            unreachable!("recorded creation class")
        };
        created
    }
    pub(crate) fn record(&self) -> &Arc<DecodedWorldRecord> {
        &self.record
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldCreationRelease {
    intent: AdmittedWorldCreationIntent,
    position: u64,
    digest: [u8; 32],
}
impl AdmittedWorldCreationRelease {
    pub(super) fn from_validated_journal(
        intent: AdmittedWorldCreationIntent,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        Self {
            intent,
            position,
            digest,
        }
    }
    pub(crate) fn intent(&self) -> &AdmittedWorldCreationIntent {
        &self.intent
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}
