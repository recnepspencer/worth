//! Recovered opens keep their exact decoded allocation through custody escape.
use crate::durability::{DecodedWorldRecord, WorldOpenAttemptRecord, WorldRecord};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct AdmittedWorldOpenAttempt {
    meaning: Arc<DecodedWorldRecord>,
    position: u64,
    digest: [u8; 32],
}
impl AdmittedWorldOpenAttempt {
    pub(super) fn from_validated_journal(
        meaning: Arc<DecodedWorldRecord>,
        position: u64,
        digest: [u8; 32],
    ) -> Self {
        assert!(matches!(&meaning.record, WorldRecord::OpenAttempt(_)));
        Self {
            meaning,
            position,
            digest,
        }
    }
    pub(crate) fn open(&self) -> &WorldOpenAttemptRecord {
        let WorldRecord::OpenAttempt(open) = &self.meaning.record else {
            unreachable!("admitted open")
        };
        open
    }
    pub(crate) fn record(&self) -> &Arc<DecodedWorldRecord> {
        &self.meaning
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}
impl std::ops::Deref for AdmittedWorldOpenAttempt {
    type Target = WorldOpenAttemptRecord;
    fn deref(&self) -> &Self::Target {
        self.open()
    }
}
